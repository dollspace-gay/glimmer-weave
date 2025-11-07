/**
 * Simple test to verify mmap works on this system
 */
#include <stdio.h>
#include <sys/mman.h>
#include <unistd.h>

int main() {
    printf("Testing mmap...\n");
    printf("MAP_PRIVATE = %d (0x%x)\n", MAP_PRIVATE, MAP_PRIVATE);
    printf("MAP_ANONYMOUS = %d (0x%x)\n", MAP_ANONYMOUS, MAP_ANONYMOUS);
    printf("PROT_READ = %d\n", PROT_READ);
    printf("PROT_WRITE = %d\n", PROT_WRITE);

    void *ptr = mmap(NULL, 65536, PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    if (ptr == MAP_FAILED) {
        perror("mmap failed");
        return 1;
    }

    printf("mmap succeeded! Address: %p\n", ptr);

    // Try to write to it
    *(int*)ptr = 42;
    printf("Write test: %d\n", *(int*)ptr);

    munmap(ptr, 65536);
    printf("All tests passed!\n");
    return 0;
}
