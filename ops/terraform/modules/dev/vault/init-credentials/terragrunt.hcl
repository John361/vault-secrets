include "backend" {
  path = find_in_parent_folders("backend-postgres.hcl")
}

terraform {
  source = "../../../../libs/vault//init-credentials"
}

locals {
  root_vars = read_terragrunt_config(find_in_parent_folders("root.hcl"))
  env_vars  = read_terragrunt_config(find_in_parent_folders("env.hcl"))
}

inputs = {
  environment            = local.env_vars.locals.environment
  app_name               = local.env_vars.locals.app_name
  vault_init_credentials = local.env_vars.locals.vault_init_credentials
}
