terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.6.2"
    }
  }

  backend "s3" {
    bucket         = "shared-tf-state"
    region         = "ap-southeast-2"
    encrypt        = true
    dynamodb_table = "shared-tf-state-lock"
  }
}

provider "aws" {
  default_tags {
    tags = {
      Project = "Maccas API"
    }
  }
}
