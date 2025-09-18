// Security headers middleware
// Week 7: Security headers and session controls

use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};

/// Security headers middleware
pub async fn security_headers_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Content Security Policy
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline' 'unsafe-eval'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             font-src 'self' data:; \
             connect-src 'self' https:; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        ),
    );
    
    // HTTP Strict Transport Security
    headers.insert(
        "Strict-Transport-Security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );
    
    // X-Content-Type-Options
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    
    // X-Frame-Options
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("DENY"),
    );
    
    // X-XSS-Protection
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Referrer Policy
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Permissions Policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static(
            "geolocation=(), \
             microphone=(), \
             camera=(), \
             payment=(), \
             usb=(), \
             magnetometer=(), \
             gyroscope=(), \
             speaker=()"
        ),
    );
    
    Ok(response)
}
