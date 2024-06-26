#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/sha.h>
#include <sys/time.h>
#include <pthread.h>

#define INPUT_PREFIX "taxborn/cpu/"
#define INPUT_SUFFIX_LENGTH 1
#define MAX_INPUT_LENGTH (sizeof(INPUT_PREFIX) + 12)
#define HASH_LENGTH 32
#define NUM_THREADS 10 // Change this to the desired number of threads

struct ThreadData {
    unsigned long long start;
    unsigned long long end;
    unsigned char* smallest_hash;
    pthread_mutex_t* mutex;
};

void* hash_range(void* arg) {
    struct ThreadData* data = (struct ThreadData*)arg;
    unsigned long long i;
    unsigned long long hashes_computed = 0;

    for (i = data->start; i < data->end; i++) {
        char input[MAX_INPUT_LENGTH];
        snprintf(input, MAX_INPUT_LENGTH, "%s%013llu", INPUT_PREFIX, i);

        unsigned char hash[HASH_LENGTH];
        SHA256((const unsigned char*)input, strlen(input), hash);
        hashes_computed++;

        pthread_mutex_lock(data->mutex);
        if (memcmp(hash, data->smallest_hash, HASH_LENGTH) < 0) {
            memcpy(data->smallest_hash, hash, HASH_LENGTH);
            printf("Smallest hash: %s (%02X%02X%02X%02X%02X%02X%02X%02X%02X...%02X%02X)\n", input, hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7], hash[8], hash[HASH_LENGTH - 2], hash[HASH_LENGTH - 1]);
        }
        pthread_mutex_unlock(data->mutex);
    }

    return (void*)hashes_computed;
}

int main() {
    unsigned char smallest_hash[HASH_LENGTH];
    memset(smallest_hash, 0xFF, HASH_LENGTH);
    pthread_t threads[NUM_THREADS];
    struct ThreadData thread_data[NUM_THREADS];
    pthread_mutex_t mutex;
    struct timeval start_time, end_time;
    double elapsed_time;
    unsigned long long total_hashes_computed = 0;

    gettimeofday(&start_time, NULL);
    pthread_mutex_init(&mutex, NULL);

    unsigned long long range_size = 1000000000ULL / NUM_THREADS;
    for (int i = 0; i < NUM_THREADS; i++) {
        thread_data[i].start = i * range_size;
        thread_data[i].end = (i == NUM_THREADS - 1) ? 1000000000ULL : (i + 1) * range_size;
        thread_data[i].smallest_hash = smallest_hash;
        thread_data[i].mutex = &mutex;
        pthread_create(&threads[i], NULL, hash_range, &thread_data[i]);
    }

    for (int i = 0; i < NUM_THREADS; i++) {
        unsigned long long thread_hashes;
        pthread_join(threads[i], (void**)&thread_hashes);
        total_hashes_computed += thread_hashes;
    }

    pthread_mutex_destroy(&mutex);
    gettimeofday(&end_time, NULL);

    elapsed_time = (end_time.tv_sec - start_time.tv_sec) + (end_time.tv_usec - start_time.tv_usec) / 1000000.0;
    double mhs = total_hashes_computed / (elapsed_time * 1000000.0);
    printf("Total hashes computed: %llu\n", total_hashes_computed);
    printf("Total time: %.2f seconds\n", elapsed_time);
    printf("Average MH/s: %.2f\n", mhs);

    return 0;
}
