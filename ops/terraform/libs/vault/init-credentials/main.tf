# Create default credentials
resource "vault_generic_secret" "credentials" {
  for_each  = { for secret in var.vault_init_credentials : secret.path => secret }
  path      = "${var.app_name}-${var.environment}/${each.key}"
  data_json = jsonencode(each.value.data)
}
