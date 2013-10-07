Title: Problem Set 3
Date: 2013-10-07
Category: PS
Tags: Problem Sets, Web Server
Author: David Evans and Weilin Xu
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
A leap rom zhttpto to zhtta.


## Taste the fresh Rust 0.8
zhttpto in Rust 0.8
explain


https://github.com/cs4414/CS4414-Public-Reference-Solution/commit/0fe2e9ad165d01038a92656252f230f6f5013644
diff from 0.7:
* module path movement
* TCP connection API was totally changed.



## Explain the starting code
explain simple FILO scheduling


## Safe counter in need

<div class="problem">

<b>Problem 1.</b> (modify <span class="file">zhtta.rs</span>) 
<br>
Modify the zhtta code so it supports a safe visitor counter protected by Read/Write Lock. You should implement your own mutex code. Referencing the mutex API in Rust will just earn 20% of grade.
</div>

## Job scheduling methods

<div class="problem">

<b>Problem 2.</b> (modify <span class="file">answers.md</span>)
<br>
study several scheduling methods (round-robin, FIFO, LIFO, SPRT ...), and design your preferred connection scheduling policy.
assume the bottleneck as network bandwidth
1. file size - bytes sent
2. server-client network connection bandwidth (could be inferred by IP address )
</div>


## Schedule web requests
implement SPRT

Reference: 
[  ] Bianca Schroeder, Mor Harchol-Balter (CMU). Web servers under overload: How scheduling can help
[31] Mor Harchol-Balter, et al (CMU). Size-based scheduling to improve web performance. ACM Transactions on Computer Systems, 21(2), May 2003.
[54] Mayank Rawat, et al (UIC). SWIFT: Scheduling in web servers for fast reponse time. In Second IEEE International Symposium on Network Computing and Applications, April 2003.

<div class="problem">
<b>Problem 3.</b> (modify <span class="file">zhtta.rs</span>)
<br>
Modify the zhtta code to implement your preferred scheduling (in user space but not kernel). Please read the example of FILO before coding.
</div>


## GASH in zhtta

<div class="problem">
<b>Problem 4.</b> (modify <span class="file">zhtta.rs</span>)
<br>
Modify the zhtta code to integrate gash in zhtta. You may use your own gash, or use that in PS2 reference solution. How would you schedule the connection to gash among other file requests?
</div>


## C10k problem
<div class="problem">
<b>Problem 5.</b>
<br>
Use an existing tool to test and compare the performance of zhtta and zhttpto. Is there any improvement on performance? Why?
</div>


