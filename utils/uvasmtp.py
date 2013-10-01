#!/usr/bin/python
import smtplib

def send_email(recipient, subject, body):
    SMTP_SERVER = 'smtp.gmail.com'
    SMTP_PORT = 587
     
    sender = 'xuweilin@virginia.edu'
    username = "wx4ed@virginia.edu"
    password = ""
     
    headers = ["From: " + sender,
               "Subject: " + subject,
               "To: " + recipient,
               "MIME-Version: 1.0",
               "Content-Type: text/html"]
    headers = "\r\n".join(headers)
     
    session = smtplib.SMTP(SMTP_SERVER, SMTP_PORT)
     
    session.ehlo()
    session.starttls()
    session.ehlo
    session.login(username, password)
     
    session.sendmail(sender, recipient, headers + "\r\n\r\n" + body)
    session.quit()

