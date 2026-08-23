locals {
  pg_backend_schema_clean = replace(path_relative_to_include(), "/", "-")
  pg_backend_uri_env      = get_env("TG_PG_CUSTOMER_ZONE", "dev")
  pg_backend_uri          = trimspace(file("../.postgres_backend_uri_${local.pg_backend_uri_env}"))
}

generate "backend" {
  path      = "backend.tf"
  if_exists = "overwrite_terragrunt"
  contents  = <<EOF
    terraform {
      backend "pg" {
        conn_str      = "${local.pg_backend_uri}"
        schema_name   = "${local.pg_backend_schema_clean}"
      }
    }
  EOF
}
