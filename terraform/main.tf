terraform {
  backend "s3" {
    bucket   = "jakewoods-terraform-state"
    key      = "global/s3/mk-rss.tfstate"
    region   = "ap-southeast-2"
    encrypt  = true
  }
}

data "terraform_remote_state" "jakewoods_aws" {
  backend = "s3"
  config = {
    bucket  = "jakewoods-terraform-state"
    key     = "global/s3/jakewoods-aws.tfstate"
    region  = "ap-southeast-2"
    encrypt  = true
  }
}
