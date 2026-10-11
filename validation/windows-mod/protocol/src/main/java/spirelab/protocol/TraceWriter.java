package spirelab.protocol;

import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;

/**
 * Appends trace-v1 records to a file as UTF-8 JSON Lines.
 *
 * <p>Fail-closed writer contract:
 * <ul>
 *   <li>the first record appended is the {@link TraceRecord.Header};</li>
 *   <li>exactly one {@code initial} immediately after the header;</li>
 *   <li>{@code step} records strictly in 0-based sequence;</li>
 *   <li>at most one {@code end}, after which the writer is finished;</li>
 *   <li>every record after the header carries the SHA-256 fingerprint of
 *       the complete preceding line (the writer computes and enforces the
 *       chain — records cannot be appended in arbitrary order).</li>
 * </ul>
 */
public final class TraceWriter implements AutoCloseable {

    private final Path file;
    private final BufferedWriter out;

    private boolean headerWritten;
    private boolean initialWritten;
    private boolean finished;
    private long nextIndex;
    private String previousLine; // complete preceding line (no trailing newline)

    public TraceWriter(Path file) throws IOException {
        this.file = file;
        if (file.getParent() != null) Files.createDirectories(file.getParent());
        this.out = Files.newBufferedWriter(file, StandardCharsets.UTF_8);
    }

    /**
     * Append one record, enforcing the strict record order:
     * header first, exactly one initial, 0-based step sequence, a single
     * terminating end. Every record after the header carries the SHA-256
     * of the complete preceding line (computed here; never caller-supplied).
     */
    public void append(TraceRecord record) throws IOException {
        if (record == null) throw new ProtocolException("record must not be null");
        if (finished) throw new ProtocolException("writer is finished: no record may follow the end record");

        if (record instanceof TraceRecord.Header header) {
            if (headerWritten || initialWritten || nextIndex > 0) {
                throw new ProtocolException("the header must be the first record appended");
            }
            String line = header.toJsonLine(null);
            out.write(line);
            out.newLine();
            out.flush();
            headerWritten = true;
            previousLine = line;
            return;
        }
        if (!headerWritten) {
            throw new ProtocolException("the first record must be the trace header, got '" + record.kind() + "'");
        }

        if (record instanceof TraceRecord.Initial initial) {
            if (initialWritten) {
                throw new ProtocolException("only one initial record is allowed");
            }
            if (nextIndex != 0) {
                throw new ProtocolException("the initial record must come before any step");
            }
            writeWithFingerprint(initial.toJsonLine(fingerprint()));
            initialWritten = true;
            return;
        }
        if (!initialWritten) {
            throw new ProtocolException("an initial record must precede '" + record.kind() + "' records");
        }

        if (record instanceof TraceRecord.Step step) {
            if (step.index() != nextIndex) {
                throw new ProtocolException("step index " + step.index()
                    + " breaks the strict 0-based sequence (expected " + nextIndex + ")");
            }
            writeWithFingerprint(step.toJsonLine(fingerprint()));
            nextIndex++;
            return;
        }

        if (record instanceof TraceRecord.End end) {
            writeWithFingerprint(end.toJsonLine(fingerprint()));
            finished = true;
            return;
        }
        throw new ProtocolException("unknown record kind: " + record.kind());
    }

    private void writeWithFingerprint(String line) throws IOException {
        out.write(line);
        out.newLine();
        out.flush();
        previousLine = line;
    }

    private String fingerprint() {
        return sha256Hex(previousLine);
    }

    private static String sha256Hex(String line) {
        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] digest = md.digest(line.getBytes(StandardCharsets.UTF_8));
            StringBuilder sb = new StringBuilder(64);
            for (byte b : digest) {
                sb.append(String.format("%02x", b));
            }
            return sb.toString();
        } catch (java.security.NoSuchAlgorithmException e) {
            throw new IllegalStateException("SHA-256 unavailable", e);
        }
    }

    public void close() throws IOException {
        out.close();
    }

    public Path file() {
        return file;
    }
}
