# Activate backend
resource "vault_auth_backend" "backends" {
  type = "userpass"
}

# Create simple app policy
resource "vault_policy" "app-policy" {
  name   = "userpass-${var.app_name}-${var.environment}"
  policy = <<EOT
    path "${var.app_name}-${var.environment}/*" {
      capabilities = ["create", "read", "update", "delete", "list", "sudo"]
    }
  EOT
}

# Get app user
data "vault_generic_secret" "vault_admin_user" {
  path = "${var.app_name}-${var.environment}/vault/users/${var.app_name}"
}

# Create users
resource "vault_generic_endpoint" "users" {
  depends_on           = [vault_auth_backend.backends]
  path                 = "auth/userpass/users/${var.app_name}"
  ignore_absent_fields = true

  data_json = jsonencode({
    password = data.vault_generic_secret.vault_admin_user.data["password"]
    policies = [vault_policy.app-policy.name]
  })
}
