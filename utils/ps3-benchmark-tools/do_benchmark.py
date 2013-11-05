import csv
import re
import urllib
import os
import sys
import zipfile
import subprocess
import shutil


def generate_test_files(dir_name = "www"):
    if not os.path.isdir(dir_name):
        os.makedirs(dir_name)
    os.chdir(dir_name)
    
    for testfile in ['5K', '5M', '10M', '20M', '40M', '80M', '512M']:
        if not os.path.isfile(testfile+".bin"):
            os.system("dd if=/dev/urandom of=%s.bin bs=%s count=1" % (testfile, testfile))
    
    os.chdir("..")

def main():
    repo_name = 'cs4414-ps3'
    csv_path = 'ps3-responses.csv'
    tarball_dir = "ps3-code-submission"

    ''' Already downloaded submissions into Dropbox/ps3/code-submission/
    if not os.path.isfile(csv_path):
        print "No .csv file"
        return
    if not os.path.isdir(tarball_dir):
        os.makedirs(tarball_dir)
        
    csv_parse(csv_path, tarball_dir)
    '''
    
    benchmark_home_dir = os.path.split(os.path.realpath(__file__))[0]
    os.chdir(benchmark_home_dir)
    
    generate_test_files()
    
    os.chdir(benchmark_home_dir)
    if not os.path.isfile("zhtta-test-urls.txt"):
        os.system("wget http://www.cs.virginia.edu/~wx4ed/cs4414/ps3/zhtta-test-urls.txt")
        os.system("tr \"\\n\" \"\\0\" < zhtta-test-urls.txt > zhtta-test-urls.httperf")
    
    os.chdir(tarball_dir)

    for files in os.listdir("."):
        if files.endswith(".zip"):
            folder = os.path.splitext(files)[0]
            if not os.path.isdir(folder):
                archive = zipfile.ZipFile(files)
                archive.extractall(os.path.splitext(files)[0])
            os.chdir(folder)
            os.chdir(os.listdir(".")[0])

            if os.path.isfile("./zhtta.rs"):
                print "\nBENCHMARKING SUBMISSION IN %s:\n" % folder
                
                sys.stdout.flush()
                
                student_path = os.getcwd()
                
                print "Building zhtta..."
                os.system("make > /dev/null 2>&1");
                if not os.path.isfile("./zhtta"):
                    os.system("rustc ./zhtta.rs > /dev/null 2>&1")
                
                
                
                if not os.path.isfile("./zhtta"):
                    print "Could not build zhtta"
                    continue
                else:
                    print "zhtta is ready."
                    zhtta_cmd = "%s/./www/zhtta" % benchmark_home_dir
                    
                    os.system("killall zhtta")
                    os.remove(zhtta_cmd)
                    
                    shutil.copyfile('zhtta', zhtta_cmd)
                    os.system("chmod +x %s" % zhtta_cmd)
                    
                    os.chdir(benchmark_home_dir)
                    os.chdir("www")
                    os.system("./zhtta > %s 2>&1 &" % ("../output/"+folder.split()[0]+'-zhtta.output'))
                    print "wait zhtta for 3 seconds..."
                    import time
                    time.sleep(3)

                print "BEGINNING HTTPERF\n"
                sys.stdout.flush()
                
                os.chdir(benchmark_home_dir)
                
                print "Httperf round1 - concurrency 60"
                httperfoutput_name = "%s-httperf-output-round1.txt" % folder.split()[0]
                httperferroutput_name = "%s-httperf-err-output-round1.txt" % folder.split()[0]
                os.system("httperf --server localhost --port 4414 --rate 60 --num-conns 60 --wlog=y,./zhtta-test-urls.httperf 2>./output/%s 1>./output/%s" % (httperferroutput_name, httperfoutput_name))
                
                sys.stdout.flush()
                sys.stderr.flush()
                
                print "Httperf round2 - concurrency 60"
                httperfoutput_name = "%s-httperf-output-round2.txt" % folder.split()[0]
                httperferroutput_name = "%s-httperf-err-output-round2.txt" % folder.split()[0]
                os.system("httperf --server localhost --port 4414 --rate 60 --num-conns 60 --wlog=y,./zhtta-test-urls.httperf 2>./output/%s 1>./output/%s" % (httperferroutput_name, httperfoutput_name))
                
                sys.stdout.flush()
                sys.stderr.flush()
                
                print "Httperf round3 - concurrency 300"
                
                httperfoutput_name = "%s-httperf-output-round3.txt" % folder.split()[0]
                httperferroutput_name = "%s-httperf-err-output-round3.txt" % folder.split()[0]
                
                os.system("httperf --server localhost --port 4414 --rate 300 --num-conns 300 --wlog=y,./zhtta-test-urls.httperf 2>./output/%s 1>./output/%s" % (httperferroutput_name, httperfoutput_name))
                
                sys.stdout.flush()
                sys.stderr.flush()
                os.system("killall zhtta -s SIGINT")
                os.chdir(student_path)
            else:
                print "no zhtta.rs: %s\n" % os.getcwd()
            os.chdir("../..")


if  __name__ =='__main__':
    main()
