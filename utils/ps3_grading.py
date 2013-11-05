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
    

def delete_test_files():
    os.remove("./5K.bin")
    os.remove("./5M.bin")
    os.remove("./10M.bin")
    os.remove("./20M.bin")
    os.remove("./40M.bin")
    os.remove("./80M.bin")
    os.remove("./512M.bin")

    os.remove("./zhtta-test-urls.httperf")

def main():
    repo_name = 'cs4414-ps3'

    csv_path = 'ps3-responses.csv'
    tarball_dir = "code-submission"

    ''' Already downloaded submissions into Dropbox/ps3/code-submission/
    if not os.path.isfile(csv_path):
        print "No .csv file"
        return
    if not os.path.isdir(tarball_dir):
        os.makedirs(tarball_dir)
        
    csv_parse(csv_path, tarball_dir)
    '''
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
                print "\nBENCHMARKING SUBMISSION IN %s:\n" % os.getcwd()
                
                sys.stdout.flush()
                
                # run Count Lines Of Code
                # os.system("../../../cloc.pl --quiet zhtta.rs")
                if not os.path.isfile("./512M.bin"):
                    os.system("ln -s " + tarball_dir + "/5K.bin ./")
                    os.system("ln -s " + tarball_dir + "/5M.bin ./")
                    os.system("ln -s " + tarball_dir + "/10M.bin ./")
                    os.system("ln -s " + tarball_dir + "/20M.bin ./")
                    os.system("ln -s " + tarball_dir + "/40M.bin ./")
                    os.system("ln -s " + tarball_dir + "/80M.bin ./")
                    os.system("ln -s " + tarball_dir + "/512M.bin ./")

                os.system("cp ../../../" + tarball_dir + "/zhtta-test-urls.httperf ./")
                
                os.system("make > /dev/null 2>&1");
                if not os.path.isfile("./zhtta"):
                    os.system("rustc ./zhtta.rs > /dev/null 2>&1")
                if not os.path.isfile("./zhtta"):
                    print "Could not build zhtta"
                    continue
                else:
                    os.system("./zhtta > /dev/null 2>&1 &")

                print "BEGINNING HTTPERF\n"
                sys.stdout.flush()
                proc = subprocess.Popen(["httperf","--server", "localhost", "--port", "4414", "--rate", "60", "--num-conns", "60", "--wlog=y,./zhtta-test-urls.httperf"], stdout=subprocess.PIPE)
                for line in iter(proc.stdout.readline,''):
                    # need test-duration(s), reply time(ms), Net I/O, errors
                    output_line = line.rstrip()
                    testduration = re.search(r'test-duration (\d+\.\d+) s', output_line)
                    replytime = re.search(r'Reply time [ms]: response (\d+\.\d+)', output_line)
                    netio = re.search(r'Net I/O: (\d+\.\d+) KB/s', output_line)
                    errorcount = re.search(r'Errors: total (\d+)', output_line)
                    if testduration:
                        print "Test duration: %s s\n" % testduration.group(1)
                    elif replytime:
                        print "Reply time: %s ms\n" % replytime.group(1)
                    elif netio:
                        print "Net I/O: %s KB/s\n" % netio.group(1)
                    elif errorcount:
                        print "Error count: %s\n" % errorcount.group(1)    
                    sys.stdout.flush()

                print "END HTTPERF\n"
                sys.stdout.flush()
                os.system("killall zhtta")
                delete_test_files()
            else:
                print "no zhtta.rs: %s\n" % os.getcwd()
            os.chdir("../..")


if  __name__ =='__main__':main()


