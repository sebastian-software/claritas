/**
 * Claritas - Readability + Defuddle Rust Port
 *
 * Node.js bindings for the Rust implementation.
 */

// TODO: Import native bindings once built
// import { extractReadability, isProbablyReaderable } from './native';

export interface Article {
  title: string;
  byline: string | null;
  content: string;
  textContent: string | null;
  length: number;
  excerpt: string | null;
  siteName: string | null;
  dir: string | null;
  lang: string | null;
}

export interface ReadabilityOptions {
  url?: string;
}

/**
 * Extract readable content from HTML.
 */
export function extractReadability(
  html: string,
  options?: ReadabilityOptions
): Article | null {
  // TODO: Call native binding
  throw new Error('Native bindings not yet built');
}

/**
 * Check if a document is probably readable.
 */
export function isProbablyReaderable(html: string): boolean {
  // TODO: Call native binding
  throw new Error('Native bindings not yet built');
}
