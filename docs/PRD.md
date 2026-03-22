# Vira — Product Requirements

## Overview

Cross-platform Point of Sale desktop application for retail merchants. Built with Tauri v2, SvelteKit, and SQLite.

## Status

Active development — core POS functionality implemented. Receipt printing, Sola Gateway integration, and multi-user support in progress.

## Goals

- Fast, reliable point-of-sale transactions for small/medium retail
- Credit card processing via Sola Gateway (card-present and card-not-present)
- ESC/POS thermal receipt printing
- Inventory tracking with low-stock alerts
- Sales and product reports
- Multi-user support with admin/cashier roles
- Cross-platform (Windows, macOS, Linux)
- Auto-update via GitHub Releases

## Non-Goals

- Cloud sync / multi-location (future consideration)
- E-commerce / online ordering
- Accounting integration
- Barcode scanner hardware integration (beyond manual entry)

## Features

### Core POS
- Product grid with search and category filtering
- Cart management (add, remove, quantity adjust)
- Multiple payment methods (cash, check, credit card, other)
- Split payment support
- Order numbering with configurable prefix
- Draft order saving

### Payment Processing
- Sola Gateway integration for credit cards
- Card-present (terminal) and card-not-present (manual entry)
- Device selection for payment terminals
- Transaction result display with full gateway response

### Receipt Printing
- ESC/POS thermal receipt printing (80mm)
- Receipt preview before printing
- Auto-print mode
- Configurable store info, header, footer

### Inventory
- Product CRUD with SKU, barcode, images
- Stock tracking with automatic deduction on sale
- Low stock alerts with configurable thresholds
- Category management

### Customers
- Customer database with contact and billing info
- Order history per customer
- Customer assignment to orders

### Orders & Refunds
- Order list with search, status filter, date range
- Order detail with void and refund capabilities
- Partial and full refunds with restock option

### Reports
- Sales over time (daily, weekly, monthly)
- Product performance metrics
- Inventory summary

### Settings
- Store branding (name, logo, address)
- Tax configuration (rate, included-in-price)
- Payment method toggles
- Printer configuration
- Color themes and dark mode
- Employee management (admin/cashier roles)

### Auto-Update
- Tauri updater plugin with GitHub Releases
- CI/CD via GitHub Actions for all platforms
