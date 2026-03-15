# T06: Plan 06

**Slice:** S03 — **Milestone:** M001

## Description

Complete OCR pipeline by bundling ONNX models and implementing postprocessing logic to extract actual text from model outputs.

Purpose: Fix the blocker preventing OCR from returning real text (currently returns empty vectors).

Output: Working OCR engine that loads bundled NDLOCR-Lite models and extracts text with markdown formatting.
