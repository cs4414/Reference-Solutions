Title: Problem Set 3
Date: 2013-10-07
Category: PS
Tags: Problem Sets, Web Server
Author: David Evans and Weilin Xu and Purnam Jantrania
Slug: ps3

<div class="due">
Due: 11:59pm on Monday, 28 October
</div>

## Purpose

The goals of this assignment are to:

- Learn about readers-writer lock, one of the advanced mutual exclusion primitives.

- Learn about job scheduling strategies and their applicability to a variety of scenes.

- Understand how to optimize the performance of a application by implementing a simple web server.


**Collaboration Policy.** For this problem set, you are expected to
work with one other student in the class.  You may select your own
partner (but will be required to work with someone else on the next
assignment).  If you don't already have a partner, use the [Piazza
forum](https://piazza.com/virginia/fall2013/cs4414).

You and your partner should work together in a way that is efficient and
collaborative, and ensures that both of you understand everything in the
code you submit.  You could choose to use [pair
programing](http://en.wikipedia.org/wiki/Pair_programming) if you want,
but it is also fine to discuss together a plan for the assignment, work
independently on different parts of the project, then explain and do
code reviews on each others work after.

As part of the grading for this assignment, you will be a short demo
with one of the course staff, and both partners will be expected to be
able to answer questions about how your code works.

Please note that only one of you need to create the private repository
for this problem set, the other member should work in the same
repository as a collaborator.

In addition to working directly with your partner, you should feel free
to discuss the problems, provide coding help, and ask for help with any
students in the class (or anyone else in the world, for that matter), so
long as you don't to it in a way that is detrimental to your own or
anyone else's learning.  You can do this in person, using the Piazza
forum, using the `#cs4414` and `#rust` IRC channels, or any other
communication medium you find most effective.


# Getting Started

Before continuing with this assignment, you should find a teammate and 
**one of you** should:

1. Set up the private repository named 'cs4414-ps3'.
2. Add your teammate and 'cs4414uva' as the collaborators.
3. Clone the empty private repository to your working environment. Instead of _mygithubname_ below, use your github username.

```text
    git clone https://github.com/mygithubname/cs4414-ps3.git
```

4. Get the starting code for ps3.  
	
```text
    git remote add course https://github.com/cs4414/ps3.git
    git pull course master
    git push --tags origin master
```
 
After finishing these steps, everyone in the team should have access to your own `cs4414-ps3` 
repository that contains starting code for ps3.


## Background

introduce the main-stream modern Web servers: apache, nginx, 
You are not expected to build a better one in this assignment, but it's possible to build a very special one.


## Taste the fresh Rust 0.8

The latest Rust 0.8 has been released 10 days ago on Sep 26. Here's the detailed webpage: [Rust 0.8 Released](https://mail.mozilla.org/pipermail/rust-dev/2013-September/005804.html) We should upgrade our Rust environment to this latest version in PS3 for the numerous bugfixes since 0.7. Follwing this tutorial in [Piazza](https://piazza.com/class/hiuvlqlyk4925d?cid=64), you should be able to get Rust 0.8 work on your machine.

Unfortunetely, the 0.8 release also includes about 2200 changes in the language. Not surprisingly, your zhttpto code in PS1 couldn't run on Rust 0.8. But don't worry about that, we have provided a latest zhttpto written in Rust 0.8 in our [Github public repository of reference solutions](https://github.com/cs4414/Public-Reference-Solution). You could view this [commit diff page](https://github.com/cs4414/CS4414-Public-Reference-Solution/commit/0fe2e9ad165d01038a92656252f230f6f5013644) to experience the changes in Rust 0.8.

As you can see from the diff page, several modules changed their paths, and the network-related API were totally rewritten. 


## zhtta - 10^42 times better than zhttpto

In PS1, we have implemented a simple Web server named zhttpto. No matter how much grade you earned, however, it's far from an awesome Web server. With the help of Rust, zhttpto has got a good concurrency comparing the ancient web servers, but there still exists several obvious drawbacks. First, zhttpto uses an unsafe visitor counter in the code. What's worse, it exposes all of the files on your file system to web users. Moreover, it doesn't support advanced priority scheduling on the network requests. 

You will learn how to eliminate these drawbacks in this assignment and you might design and implement more interesting features. It will definitely take a leap from a simple zhttpto to an advanced zhtta, and hopefully, we have provided starting code in `zhtta.rs` to help you take the first step.


## Safe visitor counter

Visitor counter is one of the main features of zhtta, so we have no reason not to make it perfect. The most urgent challenge could be the concurrent access to the shared counter. 

For the first problem, you are required to implement your own synchronization primitive and use it to manage the concurrent access to the shared visitor counter. 

<div class="problem">

<b>Problem 1.</b> (modify <span class="file">zhtta.rs</span>) 
<br>
Modify the zhtta code so it supports a safe visitor counter managed by Readers-writer Lock. You should implement your own mutex code. Referencing the API in Rust will only earn 20% of grade.
</div>


## Scheduling strategies

The world's first web server didn't meet the problem of requests scheduling because it could just accept a connection at a time. However, zhtta could accept thousands of http requests at a time and serve them concurrently. In this way, scheduling strategies may result in various performance both in terms of server side and in terms of client side.

For the next problem, you are required to study several scheduling policies in detail.

<div class="problem">

<b>Problem 2.</b> (modify <span class="file">answers.md</span>)
<br>
Study several scheduling methods, such as round-robin, FIFO, FILO, SRPT, and list their pros and cons respectively.
</div>


## SRPT Scheduling on web requests

Shortest-Remaining-Processing-Time-First (SRPT) is a well-known preemtive scheduling algorithm in Web servers. By giving priority to short requests or those requests with short remaining time, the web server could achieve minimum average response time.

There're several academic papers about SRPT scheduling on web servers for your reference.

* Bianca Schroeder, Mor Harchol-Balter (CMU). Web servers under overload: How scheduling can help, 2002
* Mor Harchol-Balter, et al (CMU). Size-based scheduling to improve web performance. ACM Transactions on Computer Systems, 21(2), May 2003.
* Mayank Rawat, et al (UIC). SWIFT: Scheduling in web servers for fast reponse time. In Second IEEE International Symposium on Network Computing and Applications, April 2003.

Usually, the response time of a request depends on the size of requested file and the network connection condition between server and client. Network connection condition is another complicated topic, but it could be roughly inferred by IP address of clients in this assignment.

For the next problem, you are required to implement a SPRT scheduling algorithm in zhtta. You don't need to implement a full version described by academic papers, but you are encouraged to do so.

In order to help you implement scheduling algorithm in Rust, we have provided a simple FILO scheduling in starting code as an example. You can find two extra tasks to do the scheduling. One is for queueing requests, while the other is responsible for getting requests from queue and send responses. 


<div class="problem">
<b>Problem 3.</b> (modify <span class="file">zhtta.rs</span>)
<br>
Modify the zhtta code to implement your SRPT scheduling (in user space but not kernel). Please read the example of FILO before coding.
</div>


## C10k problem

An important measure of performance for a web server is how many concurrent connections it can handle. The [C10K problem](http://en.wikipedia.org/wiki/C10k_problem)
has been addressed by several modern web servers, including [nginx](http://en.wikipedia.org/wiki/Nginx) and [Microsoft IIS](http://en.wikipedia.org/wiki/Internet_Information_Services). 

<div class="problem">
<b>Problem 5.</b>
<br>
Use an existing tool to test and compare the performance of zhtta and zhttpto. Can you explain any difference in performance? For an example of a tool to measure performance, you can look
at [Apache Benchmark](http://httpd.apache.org/docs/2.2/programs/ab.html).

</div>


## GASH in zhtta (Optional)

Web servers like Apache offer the ability to run shell commands embedded in the web page.
For example, using [Apache Server-side Includes](http://httpd.apache.org/docs/current/howto/ssi.html),
you can put the following string in an HTML document to display the current date and time:

```text
<!--#exec cmd="date" -->
```

This is done by passing the commands embedded in the page to a shell to execute, and
then replacing the SSI tag with the result.

<div class="problem">
<b>Problem 4.</b> (modify <span class="file">zhtta.rs</span>)
<br>
Modify the zhtta code to integrate gash in zhtta, to run commands embedded in HTML pages. You may use your own gash, or use that in PS2 reference solution. How would you schedule the connection to gash among other file requests?
</div>


## White hat (Optional)

<div class="problem">
<b>Problem 4.</b> (modify <span class="file">zhtta.rs</span>)
<br>
Modify the zhtta code to protect against attacks.  For example, most students' code in PS1 allows requests to any file path in the file system; for example, if zhttpto is running in the '/home/user/ps1' folder, a request to 'http://DOMAIN/../../../etc/shadow' would send the server's list of users to the requester. We have a simple patch in the PS1 reference solution, but it's still exploitable. Research the vulnerabities and address them in your implementation.
</div>


### Submission and Demos

Once you decide to submit your project for grading after commiting some
code and documents, you should add a tag on your code repository with a
version number, and submit your assignment by providing the
corresponding URL using the form (not yet posted) for PS3.  

In addition to submitting using the form, you will also schedule a demo
at which you will present your (hopefully working!) gash shell to one of
the course staff and answer questions about how you did it.  Both team
members are expected to be able to answer questions about how you
implemented your shell at the demos.

[Submission Form](not yet posted)  
[Scheduling Demos](not yet posted)
