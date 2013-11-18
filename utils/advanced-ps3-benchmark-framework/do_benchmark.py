# TODO list:

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

def run_cmd_bg(cmdpath, params, output_file_path = None):
    if not os.path.isfile(cmdpath):
        print "%s doesn't exist." % cmdpath
        return False
                
    cmd_name = cmdpath.split("/")[-1]
    print cmd_name
    
    os.system("killall %s  > /dev/null 2>&1" % cmd_name)
    
    if output_file_path != None:
        os.system("./zhtta %s > %s 2>&1 &" % (params, output_file_path))
    else:
        os.system("./zhtta %s &" % params)
    return True

def main():
    #repo_name = 'cs4414-ps3'
    #csv_path = 'ps3-responses.csv'
    www_dir = 'www'
    code_dir = "src"
    output_dir = "original-output"
    detailed_test = False

    ''' Already downloaded submissions into Dropbox/ps3/code-submission/
    if not os.path.isfile(csv_path):
        print "No .csv file"
        return
    if not os.path.isdir(tarball_dir):
        os.makedirs(tarball_dir)
        
    csv_parse(csv_path, tarball_dir)
    '''
    
    if not os.path.isdir(code_dir):
        print "I couldn't find your code in ", code_dir
        return 
    
    if not os.path.isdir(output_dir):
        os.makedirs(output_dir)
    
    benchmark_home_dir = os.path.split(os.path.realpath(__file__))[0]
    os.chdir(benchmark_home_dir)
    
    generate_test_files(www_dir)
    
    os.chdir(benchmark_home_dir)
    if not os.path.isfile("zhtta-test-urls.txt"):
        os.system("wget http://www.cs.virginia.edu/~wx4ed/cs4414/ps3/zhtta-test-urls.txt")
        os.system("tr \"\\n\" \"\\0\" < zhtta-test-urls.txt > zhtta-test-urls.httperf")

    for dirpath in os.listdir(code_dir):
        print "Entering", dirpath
        folder = os.path.splitext(dirpath)[0]
        ## Update your version filter here
        '''
        if folder != "zhtta-v1":
            print "skip this version: ", folder
            continue
        
        
        if folder == "zhtta-starting-code":
            print "skip this version: ", folder, "TOO SLOW!!"
            continue
        if folder == "zhtta-v4":
            print "skip this version: ", folder, "the same as zhtta-v3!!"
            continue
            
        if folder != "zhtta-v1":
            print "skip this version: ", folder
            continue
        '''
        
        if folder != "zhtta":
            print "skip this version: ", folder, "the same as zhtta-v3!!"
            continue
        
        os.chdir(os.path.join(os.path.join(benchmark_home_dir, code_dir), dirpath))
        print "Building zhtta..."
        os.system("make > /dev/null 2>&1")
        
        if not os.path.isfile("./zhtta"):
            print "Could not build zhtta, skip..."
            continue
                    
        print "zhtta is ready."
        
        zhtta_path = os.path.join(os.getcwd(), "zhtta")
        
        if detailed_test ==True and folder == "zhtta-v1": # different buffer sizes.
            # Update your counter if you would like to do more test rounds based on the previous tests.
            round_count = 0
            #for buf_size in [1024*1024]:
            for buf_size in [1*1024, 5*1024, 10*1024, 50*1024, 100*1024, 500*1024, 1024*1024, 5*1024*1024, 10*1024*1024, 50*1024*1024, 100*1024*1024, 512*1024*1024]:
                round_count += 1
                
                params = "--bufsize %d " % buf_size
                params += "--dir %s" % os.path.join(benchmark_home_dir, www_dir)
                output_file_name = "%s-buffer%sKB-zhtta.output" % (folder.split()[0], buf_size/1024)
                output_file_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, output_file_name))
                run_cmd_bg(zhtta_path, params, output_file_path)
            
                print "wait zhtta for 3 seconds..."
                import time
                time.sleep(3)
            
                print "BEGINNING HTTPERF"
                sys.stdout.flush()
            
                
                print "Httperf round%d - buf size %dKB" % (round_count, buf_size/1024)
                request_list_path = os.path.join(benchmark_home_dir, "zhtta-test-urls.httperf")
                httperf_output_name = "%s-buffer%sKB-httperf-output-round%d.txt" % (folder.split()[0], buf_size/1024, round_count)
                httperf_output_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, httperf_output_name))
                httperf_err_output_name = "%s-buffer%sKB-httperf-err-output-round%d.txt" % (folder.split()[0], buf_size/1024, round_count)
                httperf_err_output_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, httperf_err_output_name))
                
                if not os.path.isfile(request_list_path):
                    print "no input list for httperf, skip..."
                    continue
                #print "httperf --server localhost --port 4414 --rate 60 --num-conns 60 --wlog=y,%s 2>%s 1>%s" % (request_list_path, httperf_err_output_path, httperf_output_path)
                os.system("httperf --server localhost --port 4414 --rate 300 --num-conns 300 --wlog=y,%s 2>%s 1>%s" % (request_list_path, httperf_err_output_path, httperf_output_path))
                
                sys.stdout.flush()
                sys.stderr.flush()
                os.system("killall zhtta -s SIGINT")
        elif detailed_test ==True and folder == "zhtta-v2": # different concurrency
            # Update your counter if you would like to do more test rounds based on the previous tests.
            round_count = 0
            #for concurrency in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024]:
            #for concurrency in [2048, 4096, 8192, 16384]:
            #for concurrency in [32768, 65536]:
            for concurrency in [1048576, 2097152]:
                round_count += 1
                
                params = "--concurrency %d " % concurrency
                params += "--dir %s" % os.path.join(benchmark_home_dir, www_dir)
                output_file_name = "%s-concurrency%d-zhtta.output" % (folder.split()[0], concurrency)
                output_file_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, output_file_name))
                run_cmd_bg(zhtta_path, params, output_file_path)
            
                print "wait zhtta for 3 seconds..."
                import time
                time.sleep(3)
            
                print "BEGINNING HTTPERF"
                sys.stdout.flush()
            
                
                print "Httperf round%d - concurrency %d" % (round_count, concurrency)
                request_list_path = os.path.join(benchmark_home_dir, "zhtta-test-urls.httperf")
                httperf_output_name = "%s-concurrency%d-httperf-output-round%d.txt" % (folder.split()[0], concurrency, round_count)
                httperf_output_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, httperf_output_name))
                httperf_err_output_name = "%s-concurrency%d-httperf-err-output-round%d.txt" % (folder.split()[0], concurrency, round_count)
                httperf_err_output_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, httperf_err_output_name))
                
                if not os.path.isfile(request_list_path):
                    print "no input list for httperf, skip..."
                    continue
                #print "httperf --server localhost --port 4414 --rate 60 --num-conns 60 --wlog=y,%s 2>%s 1>%s" % (request_list_path, httperf_err_output_path, httperf_output_path)
                os.system("httperf --server localhost --port 4414 --rate 300 --num-conns 300 --wlog=y,%s 2>%s 1>%s" % (request_list_path, httperf_err_output_path, httperf_output_path))
                
                sys.stdout.flush()
                sys.stderr.flush()
                os.system("killall zhtta -s SIGINT")
        
        
        else: # simple test
            params = "--dir %s" % os.path.join(benchmark_home_dir, www_dir)
            output_file_name = "%s-zhtta.output" % (folder.split()[0])
            output_file_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, output_file_name))
            run_cmd_bg(zhtta_path, params, output_file_path)
            
            print "wait zhtta for 3 seconds..."
            import time
            time.sleep(3)
        
            print "BEGINNING HTTPERF"
            sys.stdout.flush()
            
            test_rounds = {}
            test_rounds[1] = {'rate': 60, 'reqs': 60}
            test_rounds[2] = {'rate': 60, 'reqs': 60}
            test_rounds[3] = {'rate': 500, 'reqs': 1000}
            
            for round_num in [1, 2, 3]:
                requests_rate = test_rounds[round_num]['rate']
                requests_num = test_rounds[round_num]['reqs']
                print "Httperf round%d - rate: %d, requests %d" % (round_num, requests_rate, requests_num)
                
                request_list_path = os.path.join(benchmark_home_dir, "zhtta-test-urls.httperf")
                httperf_output_name = "%s-httperf-output-round%d.txt" % (folder.split()[0], round_num)
                httperf_output_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, httperf_output_name))
                httperf_err_output_name = "%s-httperf-err-output-round%d.txt" % (folder.split()[0], round_num)
                httperf_err_output_path = os.path.join(benchmark_home_dir, os.path.join(output_dir, httperf_err_output_name))
                
                if not os.path.isfile(request_list_path):
                    print "no input list for httperf, skip..."
                    continue
                #print "httperf --server localhost --port 4414 --rate 60 --num-conns 60 --wlog=y,%s 2>%s 1>%s" % (request_list_path, httperf_err_output_path, httperf_output_path)
                os.system("httperf --server localhost --port 4414 --rate %d --num-conns %d --wlog=y,%s 2>%s 1>%s" % (requests_rate, requests_num, request_list_path, httperf_err_output_path, httperf_output_path))
                
                sys.stdout.flush()
                sys.stderr.flush()
            os.system("killall zhtta -s SIGINT")

if  __name__ =='__main__':
    main()
