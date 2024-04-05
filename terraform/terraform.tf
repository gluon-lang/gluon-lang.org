terraform {
  backend "s3" {
    bucket = "gluon-lang-terraform"
    key    = "gluon-lang-org"
    region = "us-east-1"
  }
  required_providers {
    aws = {
      source = "hashicorp/aws"
      version = "~> 3"
    }
  }
}
