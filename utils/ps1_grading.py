# TODO list:
# x Fetch studetns' tarballs
# x Some fault tolerance
# + Extract answer.md and zhttpto.rs
# + Use cloc to count the code lines

import csv
import re
import urllib
import os
import zipfile
from subprocess import call

url_pattern = re.compile(r'https://github.com/([a-zA-Z0-9-]+)/([a-zA-Z0-9-]+)/(releases/tag/|tree/)([vV]?[0-9\.]+)')

def download_with_auth(url, filename):
    import urllib2
    print "downloading %s" % filename
    opener = urllib2.build_opener()
    opener.addheaders.append(('Cookie', 'tracker=direct; tz=America%2FNew_York; spy_repo=ferristseng%2Fcs4414-ps1; spy_repo_at=2013-09-15T21%3A00%3A59.450Z; __utma=1.1370771186.1377548854.1379274281.1379278858.13; __utmb=1.2.10.1379278858; __utmc=1; __utmz=1.1378316186.7.2.utmcsr=google|utmccn=(organic)|utmcmd=organic|utmctr=(not%20provided); user_session=_gV3EHAa4ZUQoA6vbuaHxjNDOJ2Vn6yy7MCTxI9ZxB-1RP9-; spy_user=cs4414uva; logged_in=yes; dotcom_user=cs4414uva; _gh_sess=BAh7CzoJdXNlcmkDTf1QOhBmaW5nZXJwcmludEkiJTFjNjgzNDcxNjhlYzMxMDk2MThkMTU0YzlhZDVmMmU4BjoGRVQ6Dmxhc3Rfc3Vkb0l1OglUaW1lDTFhHIAuUwSHCToNbmFub19udW1pAoUCOg1uYW5vX2RlbmkGOg1zdWJtaWNybyIHZFA6C29mZnNldGn%2BkJ06D3Nlc3Npb25faWQiJWExZmIzN2RiMzkzM2NmNDljODFkZTA5MzE3ZjNmNzQwOhBfY3NyZl90b2tlbkkiMXBBUFZ3enlQS0ZEUFZLd3JGV3pxTHFReEcvTkJsKzlFSDczWkQyTEN1TGc9BjsHRjoMY29udGV4dEkiBi8GOwdG--f632ad9e4cf92fb137c768dad2d097f609c3d338'))
    f = opener.open(url)
    data = f.read()
    with open(filename, "wb") as code:
        code.write(data)
        
def csv_parse(csv_path, tarball_dir):
    with open(csv_path, 'rb') as csvfile:
        spamreader = csv.reader(csvfile)
        for row in spamreader:
            ts, stu_name, computing_id, tag_url, q1, q2= row[0], row[1], row[2], row[3], row[6], row[7]
            tag_match = url_pattern.match(tag_url)
            if tag_match:
                github_username, repo_name, submit_version = tag_match.group(1, 2, 4)
                zip_src_tarball_url = "https://github.com/%s/%s/archive/%s.zip" % (github_username, repo_name, submit_version)
                
                tarball_name = "%s - ps1 - %s.zip" % (computing_id, submit_version)
                download_with_auth(zip_src_tarball_url, os.path.join(tarball_dir, tarball_name))
                
                if repo_name != REPO_NAME:
                    print "Student name: %s (%s)" % (stu_name, computing_id)
                    print "[ignore] Repo name error: %s\n" % repo_name
            else:
                print "Student name: %s (%s)" % (stu_name, computing_id)
                print "URL error: %s\n" % tag_url


def main():
    repo_name = 'cs4414-ps1'
    csv_path = './cs4414- PS1 Submission (Responses) - Form Responses.csv'
    tarball_dir = "code-submission"


    if not os.path.isfile(csv_path):
        print "No .csv file"
        return
    if not os.path.isdir(tarball_dir):
        os.makedirs(tarball_dir)
        
#    csv_parse(csv_path, tarball_dir)
    os.chdir(tarball_dir)

    for files in os.listdir("."):
        if files.endswith(".zip"):
            folder = os.path.splitext(files)[0]
            if not os.path.isdir(folder):
                archive = zipfile.ZipFile(files)
                archive.extractall(os.path.splitext(files)[0])
            os.chdir(folder)
            os.chdir(os.listdir(".")[0])

            if os.path.isfile("./zhttpto.rs"):
                os.system("../../../cloc.pl zhttpto.rs")
            else:
                print "no zhttpto.rs: %s\n" % os.getcwd()
            os.chdir("../..")

if  __name__ =='__main__':main()


