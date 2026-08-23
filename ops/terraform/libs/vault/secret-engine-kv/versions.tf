terraform {
  required_version = ">= 1.14.2"

  required_providers {
    vault = {
      source  = "hashicorp/vault"
      version = "5.1.0"
    }
    local = {
      source  = "hashicorp/local"
      version = "2.5.3"
    }
  }
}
