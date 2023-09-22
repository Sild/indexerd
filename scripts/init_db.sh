#!/bin/bash -e

db_host="127.0.0.1"
db_port="32306"
username="dev-user"
db_name="indexerd_dev_db"
export MYSQL_PWD="dev-password"

if [ "${1}" = "test" ]; then
  db_host="${INDEXERD_DB_HOST}"
  db_port="${INDEXERD_DB_PORT}"
  username="${INDEXERD_DB_USER}"
  db_name="${INDEXERD_DB_NAME}"
  export MYSQL_PWD="${INDEXERD_DB_PASS}"
fi

# init db -- exec command
mysql_exec="mysql --host=${db_host} -P ${db_port} -u${username} --verbose"

# create db
${mysql_exec} "-e CREATE DATABASE IF NOT EXISTS ${db_name};"
mysql_exec="${mysql_exec} ${db_name} -e"

# create tables

# campaign
${mysql_exec} "DROP TABLE IF EXISTS campaign; CREATE TABLE campaign (id int, name varchar(255), package_id int);"
${mysql_exec} "INSERT INTO campaign (id, name, package_id) VALUES (1, 'cmp_1', 2), (2, 'cmp_2', 3);"
${mysql_exec} "UPDATE campaign SET package_id=3 where id = 1;"
${mysql_exec} "DELETE from campaign where id = 1;"

# package
${mysql_exec} "DROP TABLE IF EXISTS package; CREATE TABLE package (id int, name varchar(255));"
${mysql_exec} "INSERT INTO package (id, name) VALUES (1, 'pkg_1'), (2, 'pkg_2'), (3, 'pkg_3');"

# pad
${mysql_exec} "DROP TABLE IF EXISTS pad;"
${mysql_exec} "CREATE TABLE pad (id int, name varchar(255));"

# pad_relation
${mysql_exec} "DROP TABLE IF EXISTS pad_relation;"
${mysql_exec} "CREATE TABLE pad_relation (id int, pad_id int, parent_pad_id int);"

# targeting_pad
${mysql_exec} "DROP TABLE IF EXISTS targeting_pad;"
${mysql_exec} "CREATE TABLE targeting_pad (id int, object_id int, object_type varchar(255), positive bool);"
