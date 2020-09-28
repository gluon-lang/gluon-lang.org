resource "aws_ecs_cluster" "gluon-lang" {
  name = "gluon-lang"
}

resource "aws_vpc" "aws-vpc" {
  cidr_block = "10.0.0.0/16"
  enable_dns_hostnames = true
}

resource "aws_subnet" "gluon-lang" {
  vpc_id = aws_vpc.aws-vpc.id
  cidr_block = aws_vpc.aws-vpc.cidr_block
  map_public_ip_on_launch = true
}

resource "aws_internet_gateway" "gluon-lang" {
  vpc_id = aws_vpc.aws-vpc.id
}

resource "aws_route_table" "gluon-lang" {
  vpc_id = aws_vpc.aws-vpc.id
}

resource "aws_route" "default" {
  route_table_id = aws_route_table.gluon-lang.id
  destination_cidr_block = "0.0.0.0/0"
  gateway_id = aws_internet_gateway.gluon-lang.id
}

resource "aws_route_table_association" "gluon-lang" {
  subnet_id = aws_subnet.gluon-lang.id
  route_table_id = aws_route_table.gluon-lang.id
}
