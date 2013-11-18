import re
import os
import xlwt

def parse_httperf_output(filename):
    dur = -1
    avg_resp = -1
    io = -1
    err = -1
    
    httperfoutput = open(filename)
    for line in httperfoutput.readlines():
        # need test-duration(s), reply time(ms), Net I/O, errors
        output_line = line.rstrip()
        testduration = re.search(r'test-duration (\d+\.\d+) s', output_line)
        replytime = re.search(r'Reply time \[ms\]: response (\d+\.\d+) .*', output_line)
        netio = re.search(r'Net I/O: (\d+\.\d+) KB/s', output_line)
        errorcount = re.search(r'Errors: total (\d+)', output_line)
        if testduration:
            #print "Test duration: %f s\n" % float(testduration.group(1))
            dur = float(testduration.group(1))
        elif replytime:
            #print "Reply time: %f ms\n" % float(replytime.group(1))
            avg_resp = float(replytime.group(1))
        elif netio:
            #print "Net I/O: %f MB\n" % float(netio.group(1)) * dur / 1024
            io = float(netio.group(1)) * dur / 1024
        elif errorcount:
            #print "Error count: %d\n" % int(errorcount.group(1))
            err = int(errorcount.group(1))
    '''
    print "Test duration: %f s" % dur
    print "Reply time: %f ms" % avg_response
    print "Net I/O: %f MB" % io
    print "Error count: %d" % err
    
    print "END HTTPERF\n"
    '''
    return dur, avg_resp, io, err

def add_title_to_ws(ws, line=0):
    ws.write(line, 0, "Computing_id")
    ws.write(line, 1, "Test round")
    ws.write(line, 2, "Duration Time (s)")
    ws.write(line, 3, "Average Response Time (ms)")
    ws.write(line, 4, "Net IO: (MB)")
    ws.write(line, 5, "Error count")
    ws.write(line, 6, "Buffer size (KB)")

def add_record_to_ws(ws, line, cid, rnd, dur, avg_resp, net_io, err):
    ws.write(line, 0, cid)
    ws.write(line, 1, rnd)
    ws.write(line, 2, dur)
    ws.write(line, 3, avg_resp)
    ws.write(line, 4, net_io)
    ws.write(line, 5, err)


def main():
    full_path = []
    output_dir = 'original-output'
    
    if not os.path.isdir(output_dir):
        print "I couldn't find your original result in ", code_dir
        return
    
    file_list = os.walk(output_dir)
    
    wb = xlwt.Workbook()
    ws = wb.add_sheet('ps3 benchmark')
    add_title_to_ws(ws)
    
    for root, dirs, files in file_list:
        for f in files: 
            #print f
            #print os.path.join(root, f)
            full_path.append(os.path.join(root, f))
    
    line = 1
    
    for file_name in full_path:
        report_name = file_name.split('/')[-1]
        #print res_name
        
        res = re.search(r'([\w-]+)-httperf-output-round([\d]+)\.txt', report_name)
        # concurrency test
        #res = re.search(r'([\w-]+)-concurrency(\d+)-httperf-output-round([\d]+)\.txt', report_name)
        #res = re.search(r'([\w-]+)-buffer(\d+)KB-httperf-output-round([\d]+)\.txt', report_name)
        if not res:
            continue
        computing_id = res.group(1)
        #buf_size = res.group(2)
        #concurrency = int(res.group(2))
        round_num = int(res.group(2))
        dur, avg_resp, net_io, err = parse_httperf_output(os.path.join(output_dir,report_name))
        
        add_record_to_ws(ws, line, computing_id, round_num, dur, avg_resp, net_io, err)
        line += 1

    wb.save('result-various-buffer-size.xls')
    #wb.save('buffer-size-results.xls')
        
        
if __name__ == '__main__':
    main()
