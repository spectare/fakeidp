#!/bin/bash

curl -d @search2.json -H "Content-Type: application/json" -XPOST http://`hostname -f`:8080/scim/EduUsers/.search -v

# curl -d @create1.json -H "Content-Type: application/json" -XPOST http://`hostname -f`:8080/scim/EduUsers -v

#curl http://`hostname -f`:8080/dboper/clean
#curl http://`hostname -f`:8080/dboper/count
