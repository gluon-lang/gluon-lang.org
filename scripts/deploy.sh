#!/bin/bash

ssh -i ~/Dropbox/gluon-lang.org.pem ec2-user@ec2-52-28-135-57.eu-central-1.compute.amazonaws.com \
	'(cd try_gluon && ./scripts/update.sh | tee ../build)'
