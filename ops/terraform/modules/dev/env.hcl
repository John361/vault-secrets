locals {
  ###################
  ##### GENERAL #####
  ###################
  environment = "dev"
  app_name    = "vault-secrets"


  ################
  ##### APPS #####
  ################
  # Vault
  vault_init_credentials = jsondecode(file(format("%s/ops/terraform/utils/json/vault-init-credentials-%s.json", get_repo_root(), local.environment)))
}
