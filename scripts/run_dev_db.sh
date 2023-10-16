#!/bin/bash

#!/bin/bash
container_name="indexerd-db"
docker rm -f ${container_name}
docker run --name ${container_name} \
	-e MYSQL_ROOT_PASSWORD='root-password' \
	-e MYSQL_DATABASE='indexerd_dev_db' \
	-e MYSQL_USER='dev-user' \
	-e MYSQL_PASSWORD='dev-password' \
	-p 32306:3306 \
	-d mysql:latest

sleep 10 # waiting for start
docker exec ${container_name} bash -c "MYSQL_PWD='root-password' mysql --verbose -uroot -P 3306 -e \"GRANT REPLICATION SLAVE ON *.* TO 'dev-user';\""
docker exec ${container_name} bash -c "MYSQL_PWD='root-password' mysql --verbose -uroot -P 3306 -e \"GRANT REPLICATION CLIENT ON *.* TO 'dev-user';\""
docker exec ${container_name} bash -c "MYSQL_PWD='root-password' mysql --verbose -uroot -P 3306 -e \"SET @@GLOBAL.ENFORCE_GTID_CONSISTENCY = WARN; SET @@GLOBAL.ENFORCE_GTID_CONSISTENCY = ON; SET @@GLOBAL.GTID_MODE = OFF_PERMISSIVE;SET @@GLOBAL.GTID_MODE = ON_PERMISSIVE;SET @@GLOBAL.GTID_MODE = ON\";"
