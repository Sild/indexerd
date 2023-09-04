#!/bin/bash -e

db_host="127.0.0.1"
db_port=32769
username="root"
export MYSQL_PWD="password"
db_name="test_db"

# init db/exec string
mysql_exec="mysql --host=${db_host} -P ${db_port} -u${username} --verbose"
${mysql_exec} "-e CREATE DATABASE IF NOT EXISTS ${db_name};"
mysql_exec="${mysql_exec} ${db_name} -e"

# campaign
${mysql_exec} "DROP TABLE IF EXISTS campaign;"
${mysql_exec} "CREATE TABLE campaign (id int, name varchar(255), package_id int);"