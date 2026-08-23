# Create kv-v2 engine
resource "vault_mount" "kvv2" {
  path = "${var.app_name}-${var.environment}"
  type = "kv" # -v2
  options = {
    version = "2"
    type    = "kv-v2"
  }
}
