# Authentication Redesign

## Overview
Migrate from session-based auth to JWT tokens with refresh token rotation.

## Architecture
- Access tokens: 15min TTL
- Refresh tokens: 7 day TTL with rotation
- Token storage: Keychain on iOS, HttpOnly cookies on web
