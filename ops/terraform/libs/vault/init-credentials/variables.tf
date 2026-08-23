variable "environment" {
  type        = string
  description = "Environment (dev | qa | demo | academy | preprod | prod | cloud | common | admin)"

  validation {
    condition     = can(regex("^(dev|qa|demo|academy|preprod|prod|cloud|common|admin)$", var.environment))
    error_message = "Only dev | qa | demo | academy | preprod | prod | cloud | common | admin are authorized"
  }
}

variable "app_name" {
  type        = string
  description = "App name"
}

variable "vault_address" {
  type        = string
  description = "Vault address"
  default     = "http://127.0.0.1:8200"
}

variable "vault_init_credentials" {
  type = list(object({
    path = string
    data = any
  }))
  description = "Vault init json data"
  # sensitive   = true // Issue: cannot loop on sensitive variable
}
