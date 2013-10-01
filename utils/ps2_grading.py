# TODO list:
# x Fetch studetns' tarballs
# x Some fault tolerance
# + Extract answer.md and zhttpto.rs
# + Use cloc to count the code lines

import csv
import re
import urllib
import os
import sys
import zipfile
import subprocess
import copy
from uvasmtp import send_email

url_pattern = re.compile(r'https://github.com/([a-zA-Z0-9-]+)/([a-zA-Z0-9-]+)/(releases/tag/|tree/)([vV]?[0-9\.]+)')
incomplete_url_pattern = re.compile(r'https://github.com/([a-zA-Z0-9-]+)/([a-zA-Z0-9-]+).*')

REPO_NAME = "cs4414-ps2"

def download_with_auth(url, filename):
    if os.path.isfile(filename):
        print "skip %s" % filename
        return True
    import urllib2
    print "downloading %s" % filename
    opener = urllib2.build_opener()
    opener.addheaders.append(('Cookie', 'tracker=direct; tz=America%2FNew_York; spy_repo=ferristseng%2Fcs4414-ps1; spy_repo_at=2013-09-15T21%3A00%3A59.450Z; __utma=1.1370771186.1377548854.1379274281.1379278858.13; __utmb=1.2.10.1379278858; __utmc=1; __utmz=1.1378316186.7.2.utmcsr=google|utmccn=(organic)|utmcmd=organic|utmctr=(not%20provided); user_session=_gV3EHAa4ZUQoA6vbuaHxjNDOJ2Vn6yy7MCTxI9ZxB-1RP9-; spy_user=cs4414uva; logged_in=yes; dotcom_user=cs4414uva; _gh_sess=BAh7CzoJdXNlcmkDTf1QOhBmaW5nZXJwcmludEkiJTFjNjgzNDcxNjhlYzMxMDk2MThkMTU0YzlhZDVmMmU4BjoGRVQ6Dmxhc3Rfc3Vkb0l1OglUaW1lDTFhHIAuUwSHCToNbmFub19udW1pAoUCOg1uYW5vX2RlbmkGOg1zdWJtaWNybyIHZFA6C29mZnNldGn%2BkJ06D3Nlc3Npb25faWQiJWExZmIzN2RiMzkzM2NmNDljODFkZTA5MzE3ZjNmNzQwOhBfY3NyZl90b2tlbkkiMXBBUFZ3enlQS0ZEUFZLd3JGV3pxTHFReEcvTkJsKzlFSDczWkQyTEN1TGc9BjsHRjoMY29udGV4dEkiBi8GOwdG--f632ad9e4cf92fb137c768dad2d097f609c3d338'))
    try:
        f = opener.open(url)
    except:
        #print "Oops! URL invalid: %s" % url
        return False
    data = f.read()
    with open(filename, "wb") as code:
        code.write(data)
    return True
    
def csv_parse(csv_path, tarball_dir):
    with open(csv_path, 'rb') as csvfile:
        spamreader = csv.reader(csvfile)
        # buffer the dataset in dictionary
        ps2_submission = {}
        for row in spamreader:
            ts, stu1_name, stu1_computing_id, stu2_name, stu2_computing_id, demo, tag_url = row[0], row[1], row[2], row[3], row[4], row[11], row[12]
            if not ps2_submission.has_key(stu1_computing_id):
                ps2_submission[stu1_computing_id] = copy.deepcopy(row)
            else:
                print "multiple submission from %s" % stu1_computing_id
                ps2_submission[stu1_computing_id] = copy.deepcopy(row)
        
        ##################
        for (k, row) in ps2_submission.items():
            ts, stu1_name, stu1_computing_id, stu2_name, stu2_computing_id, demo, tag_url = row[0], row[1], row[2], row[3], row[4], row[11], row[12]
            tag_match = url_pattern.match(tag_url)
            if tag_match:
                github_username, repo_name, submit_version = tag_match.group(1, 2, 4)
                zip_src_tarball_url = "https://github.com/%s/%s/archive/%s.zip" % (github_username, repo_name, submit_version)
                
                tarball_name = "%s - %s - ps2 - %s.zip" % (stu1_computing_id, stu2_computing_id, submit_version)
                ret = download_with_auth(zip_src_tarball_url, os.path.join(tarball_dir, tarball_name))
                if not ret:
                    print "Student1: %s (%s) Student2: %s (%s)" % (stu1_name, stu1_computing_id, stu2_name, stu2_computing_id)
                    print "Oops! URL invalid: %s" % zip_src_tarball_url
                
                if repo_name != REPO_NAME:
                    print "Student1: %s (%s) Student2: %s (%s)" % (stu1_name, stu1_computing_id, stu2_name, stu2_computing_id)
                    print "[ignore] Repo name error: %s\n" % repo_name
            else:
                #print "Student1: %s (%s) Student2: %s (%s)" % (stu1_name, stu1_computing_id, stu2_name, stu2_computing_id)
                #print "URL error: %s\n" % tag_url
                
                #https://github.com/victor-shepardson/cs4414-ps2/archive/master.zip
                tag_match = incomplete_url_pattern.match(tag_url)
                if not tag_match:
                    print "Student1: %s (%s) Student2: %s (%s)" % (stu1_name, stu1_computing_id, stu2_name, stu2_computing_id)
                    print "URL invalid: %s" % tag_url
                    continue
                # send email to students who didn't submit correct URL
                recipient = "%s <%s@virginia.edu>, %s <%s@virginia.edu>" % (stu1_name, stu1_computing_id, stu2_name, stu2_computing_id)
                subject = "[CS4414 Notification] Github URL issue"
                body = "Dear students, <br><br> The URL you submitted \"%s\" didn't match our requirement (which looks like https://github.com/evansuva/cs4414-ps2/releases/tag/v2.1), so we take your latest commit in master branch as your submission. <br><br> Please make sure that you submit a correct URL in future assignment, thanks. <br><br> --CS4414 Team" % tag_url
                
                
                github_username, repo_name = tag_match.group(1, 2)
                submit_version = "master"
                zip_src_tarball_url = "https://github.com/%s/%s/archive/%s.zip" % (github_username, repo_name, submit_version)
                if stu2_name == "none":
                    tarball_name = "%s - ps2 - %s.zip" % (stu1_computing_id, submit_version)
                else:
                    tarball_name = "%s - %s - ps2 - %s.zip" % (stu1_computing_id, stu2_computing_id, submit_version)
                ret = download_with_auth(zip_src_tarball_url, os.path.join(tarball_dir, tarball_name))
                if not ret:
                    body = "Dear students, <br><br> The URL you submitted \"%s\" didn't match our requirement (which looks like https://github.com/evansuva/cs4414-ps2/releases/tag/v2.1), and we couldn't fetch your code even if we took your latest commit in master branch as your submission. <br><br> Please reply your correct URL ASAP and make sure that you submit a correct URL in future assignment, thanks. <br><br> --CS4414 Team" % tag_url
                    
                print "[Warning] Please uncomment the code if you want to send this email: "
                print "Recipient: ", recipient
                print "Subject: ", subject
                print "Body: ", body
                print "\n"
                ##send_email(recipient, subject, body)
                    
                if repo_name != REPO_NAME:
                    print "Student1: %s (%s) Student2: %s (%s)" % (stu1_name, stu1_computing_id, stu2_name, stu2_computing_id)
                    print "[ignore] Repo name error: %s\n" % repo_name
                
def main():
    repo_name = 'cs4414-ps2'
    csv_path = './Copy of cs4414- PS2 Submission (Responses) - Form Responses.csv'
    tarball_dir = "ps2-code-submission"

    if not os.path.isfile(csv_path):
        print "No .csv file"
        return
    if not os.path.isdir(tarball_dir):
        os.makedirs(tarball_dir)
        
    csv_parse(csv_path, tarball_dir)
    return
    
    ###################return##################
    os.chdir(tarball_dir)

    for files in os.listdir("."):
        if files.endswith(".zip"):
            folder = os.path.splitext(files)[0]
            if not os.path.isdir(folder):
                archive = zipfile.ZipFile(files)
                archive.extractall(os.path.splitext(files)[0])
            os.chdir(folder)
            os.chdir(os.listdir(".")[0])

#            if not os.path.isfile("./answers.md"):
#                print "no answers.md: %s\n" % os.getcwd()
#            if not os.path.isfile("./zhttpto.rs"):
                
            if os.path.isfile("./zhttpto.rs"):
                print "%s:\n" % os.getcwd()
                sys.stdout.flush()
                os.system("../../../cloc.pl --quiet zhttpto.rs")
            else:
                print "no zhttpto.rs: %s\n" % os.getcwd()
            os.chdir("../..")

if  __name__ =='__main__':main()


