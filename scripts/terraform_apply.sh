#!/bin/bash
set -ex

./pack.sh
cd terraform
terraform apply -auto-approve
