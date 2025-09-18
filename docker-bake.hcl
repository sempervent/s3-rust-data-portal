# BlackLake Multi-Arch Docker Build Configuration
# Week 5: Multi-architecture image builds with docker-bake.hcl

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

variable "GIT_SHA" {
  default = ""
}

variable "BUILD_DATE" {
  default = ""
}

# ===== COMMON ARGUMENTS =====
function "common_args" {
  returns = {
    "build-arg:REGISTRY" = REGISTRY
    "build-arg:IMAGE_PREFIX" = IMAGE_PREFIX
    "build-arg:VERSION" = VERSION
    "build-arg:GIT_SHA" = GIT_SHA
    "build-arg:BUILD_DATE" = BUILD_DATE
  }
}

# ===== COMMON LABELS =====
function "common_labels" {
  returns = {
    "org.opencontainers.image.title" = "BlackLake ${IMAGE_PREFIX}"
    "org.opencontainers.image.description" = "BlackLake Data Artifact Management Platform"
    "org.opencontainers.image.vendor" = "BlackLake"
    "org.opencontainers.image.version" = VERSION
    "org.opencontainers.image.created" = BUILD_DATE
    "org.opencontainers.image.revision" = GIT_SHA
    "org.opencontainers.image.source" = "https://github.com/blacklake/blacklake"
    "org.opencontainers.image.licenses" = "MIT"
  }
}

# ===== COMMON PLATFORMS =====
variable "PLATFORMS" {
  default = ["linux/amd64", "linux/arm64"]
}

# ===== CACHE CONFIGURATION =====
function "cache_config" {
  returns = {
    "cache-from" = [
      "type=gha,scope=${IMAGE_PREFIX}-${REGISTRY}",
      "type=local,src=/tmp/.buildx-cache"
    ]
    "cache-to" = [
      "type=gha,mode=max,scope=${IMAGE_PREFIX}-${REGISTRY}",
      "type=local,dest=/tmp/.buildx-cache-new,mode=max"
    ]
  }
}

# ===== API TARGET =====
target "api" {
  dockerfile = "Dockerfile.api"
  context = "."
  platforms = PLATFORMS
  args = common_args()
  labels = common_labels()
  cache-from = cache_config()["cache-from"]
  cache-to = cache_config()["cache-to"]
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/api:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/api:latest",
    "${REGISTRY}/${IMAGE_PREFIX}/api:sha-${GIT_SHA}"
  ]
  output = ["type=registry"]
}

# ===== UI TARGET =====
target "ui" {
  dockerfile = "Dockerfile.ui"
  context = "./ui"
  platforms = PLATFORMS
  args = common_args()
  labels = common_labels()
  cache-from = cache_config()["cache-from"]
  cache-to = cache_config()["cache-to"]
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/ui:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/ui:latest",
    "${REGISTRY}/${IMAGE_PREFIX}/ui:sha-${GIT_SHA}"
  ]
  output = ["type=registry"]
}

# ===== GATEWAY TARGET =====
target "gateway" {
  dockerfile = "Dockerfile.gateway"
  context = "./ops/nginx"
  platforms = PLATFORMS
  args = common_args()
  labels = common_labels()
  cache-from = cache_config()["cache-from"]
  cache-to = cache_config()["cache-to"]
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/gateway:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/gateway:latest",
    "${REGISTRY}/${IMAGE_PREFIX}/gateway:sha-${GIT_SHA}"
  ]
  output = ["type=registry"]
}

# ===== JOBRUNNER TARGET =====
target "jobrunner" {
  dockerfile = "Dockerfile.jobrunner"
  context = "."
  platforms = PLATFORMS
  args = common_args()
  labels = common_labels()
  cache-from = cache_config()["cache-from"]
  cache-to = cache_config()["cache-to"]
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/jobrunner:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/jobrunner:latest",
    "${REGISTRY}/${IMAGE_PREFIX}/jobrunner:sha-${GIT_SHA}"
  ]
  output = ["type=registry"]
}

# ===== OTEL COLLECTOR TARGET =====
target "otel-collector" {
  dockerfile = "ops/otel/Dockerfile"
  context = "./ops/otel"
  platforms = PLATFORMS
  args = common_args()
  labels = common_labels()
  cache-from = cache_config()["cache-from"]
  cache-to = cache_config()["cache-to"]
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/otel-collector:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/otel-collector:latest",
    "${REGISTRY}/${IMAGE_PREFIX}/otel-collector:sha-${GIT_SHA}"
  ]
  output = ["type=registry"]
}

# ===== MLFLOW TARGET =====
target "mlflow" {
  dockerfile = "Dockerfile.mlflow"
  context = "."
  platforms = PLATFORMS
  args = common_args()
  labels = common_labels()
  cache-from = cache_config()["cache-from"]
  cache-to = cache_config()["cache-to"]
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/mlflow:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/mlflow:latest",
    "${REGISTRY}/${IMAGE_PREFIX}/mlflow:sha-${GIT_SHA}"
  ]
  output = ["type=registry"]
}

# ===== BUILD GROUPS =====
group "dev" {
  targets = ["api", "ui"]
}

group "prod" {
  targets = ["api", "ui", "gateway"]
}

group "all" {
  targets = ["api", "ui", "gateway", "jobrunner", "otel-collector", "mlflow"]
}

group "core" {
  targets = ["api", "ui", "gateway"]
}

group "observability" {
  targets = ["otel-collector"]
}

group "ml" {
  targets = ["mlflow"]
}

# ===== LOCAL BUILD TARGETS (for development) =====
target "api-local" {
  inherits = ["api"]
  output = ["type=docker"]
  tags = [
    "blacklake-api:local",
    "blacklake-api:dev"
  ]
}

target "ui-local" {
  inherits = ["ui"]
  output = ["type=docker"]
  tags = [
    "blacklake-ui:local",
    "blacklake-ui:dev"
  ]
}

target "gateway-local" {
  inherits = ["gateway"]
  output = ["type=docker"]
  tags = [
    "blacklake-gateway:local",
    "blacklake-gateway:dev"
  ]
}

group "local" {
  targets = ["api-local", "ui-local", "gateway-local"]
}

# ===== SECURITY & PROVENANCE =====
function "security_config" {
  returns = {
    "attest" = [
      "type=sbom,generator=image",
      "type=provenance,mode=max"
    ]
    "sbom" = "generator=image"
    "provenance" = "mode=max"
  }
}

# ===== SECURE BUILD TARGETS =====
target "api-secure" {
  inherits = ["api"]
  attest = security_config()["attest"]
  sbom = security_config()["sbom"]
  provenance = security_config()["provenance"]
}

target "ui-secure" {
  inherits = ["ui"]
  attest = security_config()["attest"]
  sbom = security_config()["sbom"]
  provenance = security_config()["provenance"]
}

group "secure" {
  targets = ["api-secure", "ui-secure"]
}