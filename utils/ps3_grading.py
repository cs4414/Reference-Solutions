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

url_pattern = re.compile(r'https://github.com/([a-zA-Z0-9-]+)/([a-zA-Z0-9-]+)/(releases/tag/|tree/)([vV]?[0-9\.]+)')

def download_with_auth(url, filename):
    import urllib2
    print "downloading %s" % filename
    opener = urllib2.build_opener()
    opener.addheaders.append(('Cookie', 'logged_in=yes; dotcom_user=cs4414uva; tz=America%2FNew_York; __utma=1.1596037320.1383061983.1383066523.1383147935.3; __utmb=1.2.10.1383147935; __utmc=1; __utmz=1.1383061983.1.1.utmcsr=(direct)|utmccn=(direct)|utmcmd=(none); user_session=KKSZ4Vc6VKetzRSIVgQxXcrUPPULzUTPzMQwrS-SuUnoEnfw; spy_user=cs4414uva; _gh_sess=BAh7CjoPc2Vzc2lvbl9pZCIlMmU1NzM1NWVjMzc3NzQwOGM0YjcyMzkzZDZjNjkzOTA6DGNvbnRleHRJIgYvBjoGRUY6EF9jc3JmX3Rva2VuSSIxd25lU3dML3ltcjdhV2c4eHdKTlRnMmxvSVVTWkJ2TmtNSlEvdEQvdTRLQT0GOwdGOg1zcHlfcmVwb0kiF1N5ZG5leUgvY3M0NDE0LXBzMQY7B1Q6EHNweV9yZXBvX2F0SXU6CVRpbWUNz2ccgNNT8LYJOg1uYW5vX251bWkC7AE6DW5hbm9fZGVuaQY6DXN1Ym1pY3JvIgdJIDoLb2Zmc2V0af6QnQ%3D%3D--df901dfc656d661bd7e67b592e268d51d56899f7'))
    f = opener.open(url)
    data = f.read()
    with open(filename, "wb") as code:
        code.write(data)
        
def csv_parse(csv_path, tarball_dir):
    with open(csv_path, 'rb') as csvfile:
        spamreader = csv.reader(csvfile)
        for row in spamreader:
            ts, partner1_name, partner1_email, partner2_name, partner2_email, partner3_name, partner3_email, safe_counter_success, wahoofirst_success, sptf_success, gash_success, benchmarking_comments, memcache_success, demo_time, extra_extension, tag_url = row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8], row[9], row[10], row[11], row[12], row[13], row[14], row[15]
            tag_match = url_pattern.match(tag_url)
            if tag_match:
                github_username, repo_name, submit_version = tag_match.group(1, 2, 4)
                zip_src_tarball_url = "https://github.com/%s/%s/archive/%s.zip" % (github_username, repo_name, submit_version)
                
                try:
                    tarball_name = "%s - ps3 - %s.zip" % (partner1_email, submit_version)
                    download_with_auth(zip_src_tarball_url, os.path.join(tarball_dir, tarball_name))
                except:
                    try:
                        tarball_name = "%s - ps3 - %s.zip" % (partner2_email, submit_version)
                        download_with_auth(zip_src_tarball_url, os.path.join(tarball_dir, tarball_name))
                
                    except:
                        try:
                            tarball_name = "%s - ps3 - %s.zip" % (partner3_email, submit_version)
                            download_with_auth(zip_src_tarball_url, os.path.join(tarball_dir, tarball_name))
                        except:
                            print "Could not download repository for %s, %s, and %s" % (partner1_email, partner2_email, partner3_email)
                        
                #if repo_name != REPO_NAME:
                #    print "Student name: %s (%s)" % (partner1_name, partner1_email)
                #    print "[ignore] Repo name error: %s\n" % repo_namenn
            else:
                print "Student name: %s (%s)" % (partner1_name, partner1_email)
                print "URL error: %s\n" % tag_url

def generate_test_files():
    if not os.path.isfile("512M.bin"):
        os.system("dd if=/dev/urandom of=5K.bin bs=5K count=1")
        os.system("dd if=/dev/urandom of=5M.bin bs=5M count=1")
        os.system("dd if=/dev/urandom of=10M.bin bs=10M count=1")
        os.system("dd if=/dev/urandom of=20M.bin bs=20M count=1")
        os.system("dd if=/dev/urandom of=40M.bin bs=40M count=1")
        os.system("dd if=/dev/urandom of=80M.bin bs=80M count=1")
        os.system("dd if=/dev/urandom of=512M.bin bs=512M count=1")
    if not os.path.isfile("zhtta-test-urls.txt"):
        os.system("wget http://www.cs.virginia.edu/~wx4ed/cs4414/ps3/zhtta-test-urls.txt")
        os.system("tr \"\\n\" \"\\0\" < zhtta-test-urls.txt > zhtta-test-urls.httperf")
#    more small files of different names & sizes - name of file does not imply size
    # For localhost - request a greater number of files
    

def delete_test_files():
    os.remove("./*.bin")
    os.remove("cs4414urls.httperf")

def main():
    repo_name = 'cs4414-ps3'

    csv_path = 'ps3-responses.csv'
    tarball_dir = "code-submission"


    if not os.path.isfile(csv_path):
        print "No .csv file"
        return
    if not os.path.isdir(tarball_dir):
        os.makedirs(tarball_dir)
        
    csv_parse(csv_path, tarball_dir)
    os.chdir(tarball_dir)
    
    generate_test_files()
    
    for files in os.listdir("."):
        if files.endswith(".zip"):
            folder = os.path.splitext(files)[0]
            if not os.path.isdir(folder):
                archive = zipfile.ZipFile(files)
                archive.extractall(os.path.splitext(files)[0])
            os.chdir(folder)
            os.chdir(os.listdir(".")[0])

            if os.path.isfile("./zhtta.rs"):
                print "%s:\n" % os.getcwd()
                
                sys.stdout.flush()
                os.system("../../../cloc.pl --quiet zhtta.rs")
                
                os.system("cp " + tarball_dir + "/5K.bin ./")
                os.system("cp " + tarball_dir + "/5M.bin ./")
                os.system("cp " + tarball_dir + "/10M.bin ./")
                os.system("cp " + tarball_dir + "/20M.bin ./")
                os.system("cp " + tarball_dir + "/40M.bin ./")
                os.system("cp " + tarball_dir + "/80M.bin ./")
                os.system("cp " + tarball_dir + "/512M.bin ./")

                os.system("cp " + tarball_dir + "/zhtta-test-urls.httperf ./")
                
                os.system("make");
                os.system("./zhtta &");
                
                os.system("httperf --server localhost --port 4414 --rate 60 --num-conns 60 --wlog=y,./zhtta-test-urls.httperf")
                delete_test_files()
            else:
                print "no zhtta.rs: %s\n" % os.getcwd()
            os.chdir("../..")

if  __name__ =='__main__':main()


