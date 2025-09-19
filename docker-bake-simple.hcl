# BlackLake Simple Docker Build Configuration
# Optimized for fast local development

# ===== VARIABLES =====
variable "REGISTRY" {
  default = "ghcr.io"
}

variable "IMAGE_PREFIX" {
  default = "blacklake"
}

variable "VERSION" {
  default = "latest"
}

variable "PLATFORMS" {
  default = ["linux/amd64", "linux/arm64"]
}

# ===== LOCAL BUILD TARGETS =====
target "api-local" {
  dockerfile = "Dockerfile.api"
  context = "."
  platforms = ["linux/amd64"]
  output = ["type=docker"]
  tags = [
    "blacklake-api:local",
    "blacklake-api:dev"
  ]
}

target "ui-local" {
  dockerfile = "Dockerfile"
  context = "./ui"
  platforms = ["linux/amd64"]
  output = ["type=docker"]
  tags = [
    "blacklake-ui:local",
    "blacklake-ui:dev"
  ]
}

target "gateway-local" {
  dockerfile = "Dockerfile.gateway"
  context = "./ops/nginx"
  platforms = ["linux/amd64"]
  output = ["type=docker"]
  tags = [
    "blacklake-gateway:local",
    "blacklake-gateway:dev"
  ]
}

target "jobrunner-local" {
  dockerfile = "Dockerfile.jobrunner"
  context = "."
  platforms = ["linux/amd64"]
  output = ["type=docker"]
  tags = [
    "blacklake-jobrunner:local",
    "blacklake-jobrunner:dev"
  ]
}

# ===== BUILD GROUPS =====
group "local" {
  targets = ["api-local", "ui-local", "gateway-local"]
}

group "dev" {
  targets = ["api-local", "ui-local"]
}

group "core" {
  targets = ["api-local", "ui-local", "gateway-local"]
}

group "all" {
  targets = ["api-local", "ui-local", "gateway-local", "jobrunner-local"]
}
