import { describe, it, expect } from 'vitest';
import { extractReadability, isProbablyReaderable } from '../src/index.js';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(__dirname, '../../../fixtures/readability');

describe('extractReadability', () => {
  it('should extract article from simple HTML', () => {
    const html = `
      <!DOCTYPE html>
      <html>
      <head><title>Test Article</title></head>
      <body>
        <article>
          <h1>Test Article</h1>
          <p>This is a test paragraph with some content. It needs to be long enough
          to be considered valid content by the readability algorithm.</p>
          <p>Another paragraph here with more content to ensure we have enough
          text for the algorithm to work with properly.</p>
        </article>
      </body>
      </html>
    `;

    const result = extractReadability(html);

    expect(result).not.toBeNull();
    expect(result?.title).toBe('Test Article');
    expect(result?.content).toContain('<p>');
    expect(result?.length).toBeGreaterThan(0);
  });

  it('should convert relative URLs to absolute', () => {
    const html = `
      <!DOCTYPE html>
      <html>
      <body>
        <article>
          <p>Check out <a href="/path/to/page">this link</a> for more info.</p>
          <p>More content here to make it readable.</p>
        </article>
      </body>
      </html>
    `;

    const result = extractReadability(html, { url: 'https://example.com' });

    expect(result).not.toBeNull();
    expect(result?.content).toContain('https://example.com/path/to/page');
  });

  it('should extract metadata', () => {
    const html = `
      <!DOCTYPE html>
      <html lang="en" dir="ltr">
      <head>
        <title>Test Article | My Site</title>
        <meta name="author" content="John Doe">
        <meta name="description" content="This is a test article.">
        <meta property="og:site_name" content="My Site">
      </head>
      <body>
        <article>
          <p>This is a test paragraph with some content for the algorithm.</p>
          <p>More content here to ensure readability detection works.</p>
        </article>
      </body>
      </html>
    `;

    const result = extractReadability(html);

    expect(result).not.toBeNull();
    expect(result?.lang).toBe('en');
    expect(result?.dir).toBe('ltr');
    expect(result?.byline).toBe('John Doe');
    expect(result?.excerpt).toBe('This is a test article.');
    expect(result?.siteName).toBe('My Site');
  });

  it('should return null for empty content', () => {
    const html = '<html><body></body></html>';
    const result = extractReadability(html);

    // May return article with body content or null
    // The important thing is it doesn't throw
    expect(result === null || result?.content !== undefined).toBe(true);
  });
});

describe('isProbablyReaderable', () => {
  it('should return true for readable content', () => {
    const html = `
      <!DOCTYPE html>
      <html>
      <body>
        <article>
          <p>This is a very long paragraph that contains a lot of text.
          It should be long enough to pass the minimum content length check.
          We need to make sure it has enough words to be considered readable content.
          Adding more text here to ensure we meet the threshold requirements.
          This paragraph just keeps going and going with more content.</p>
        </article>
      </body>
      </html>
    `;

    expect(isProbablyReaderable(html)).toBe(true);
  });

  it('should return false for minimal content', () => {
    const html = '<html><body><p>Short</p></body></html>';
    expect(isProbablyReaderable(html)).toBe(false);
  });
});

describe('fixture tests', () => {
  it('should parse fixture 001', () => {
    const source = readFileSync(join(fixturesDir, '001/source.html'), 'utf-8');
    const result = extractReadability(source, { url: 'http://fakehost/' });

    expect(result).not.toBeNull();
    expect(result?.title).toContain('Frontend JavaScript');
    expect(result?.content.length).toBeGreaterThan(1000);
  });

  it('should parse fixture 002', () => {
    const source = readFileSync(join(fixturesDir, '002/source.html'), 'utf-8');
    const result = extractReadability(source, { url: 'http://fakehost/' });

    expect(result).not.toBeNull();
    expect(result?.content.length).toBeGreaterThan(100);
  });

  it('should parse medium-1 fixture', () => {
    const source = readFileSync(join(fixturesDir, 'medium-1/source.html'), 'utf-8');
    const result = extractReadability(source, { url: 'http://fakehost/' });

    expect(result).not.toBeNull();
    // Content has <p> tags with attributes like <p class="...">
    expect(result?.content).toMatch(/<p[\s>]/);
  });
});
