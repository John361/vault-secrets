storage "raft" {
  path    = "/data/vault"
  node_id = "node1"
}

listener "tcp" {
  address     = "0.0.0.0:8200"
  tls_disable = 1
}

api_addr     = "http://localhost:8200"
cluster_addr = "http://127.0.0.1:8201"
ui           = true

disable_mlock     = true
log_level         = "info"
default_lease_ttl = "24h"
max_lease_ttl     = "720h"
