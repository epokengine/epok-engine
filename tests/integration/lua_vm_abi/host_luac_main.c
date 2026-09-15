/* Command-line front end for the same host cooker `src/lua_bytecode.rs` calls,
 * so the ABI check compares the shipping cooker's output rather than a
 * re-implementation. Usage: host_luac <chunkname> <in.lua> <out.bin> <strip>. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "epok_luac.h"

static int sink(void *ctx, const void *bytes, size_t size) {
    return fwrite(bytes, 1, size, (FILE *)ctx) == size ? 0 : 1;
}

int main(int argc, char **argv) {
    char err[512] = {0};
    FILE *in, *out;
    long length;
    char *source;
    int status;
    if (argc != 5) {
        fprintf(stderr, "usage: %s <chunkname> <in.lua> <out.bin> <strip>\n", argv[0]);
        return 2;
    }
    in = fopen(argv[2], "rb");
    if (!in) { perror(argv[2]); return 2; }
    fseek(in, 0, SEEK_END);
    length = ftell(in);
    fseek(in, 0, SEEK_SET);
    source = malloc((size_t)length + 1);
    if (!source || fread(source, 1, (size_t)length, in) != (size_t)length) {
        fprintf(stderr, "could not read %s\n", argv[2]);
        return 2;
    }
    source[length] = '\0';
    fclose(in);
    out = fopen(argv[3], "wb");
    if (!out) { perror(argv[3]); return 2; }
    status = epok_luac_cook(argv[1], source, (size_t)length, atoi(argv[4]), out, sink, err,
                            sizeof err);
    fclose(out);
    if (status != 0) {
        fprintf(stderr, "cook failed: %s\n", err);
        return 1;
    }
    return 0;
}
