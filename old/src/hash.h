#ifndef _HASH_H_
#define _HASH_H_

#include <stdint.h>

uint32_t memhash(void *p, size_t sz);	// fnv-1a hash
uint32_t memcrc(void *p, size_t sz);	// crc32

// map

// TODO: way to iterate through a map??
typedef int (*compareFunc_t)(void *, void *, size_t);
typedef uint32_t (*hashFunc_t)(void *, size_t);

struct bucketNode {
	size_t keysz;
	void *key;

	size_t valsz;
	void *val;

	struct bucketNode *next;
};

// TODO: remove
struct bucket {
	size_t len;

	struct bucketNode *head;
};

typedef struct {
	compareFunc_t cmp;
	hashFunc_t hash;

	size_t len;
	//size_t unit;

	size_t entries;

	struct bucket *buckets;
} map_t;

map_t *map_alloc(size_t len, compareFunc_t cmp, hashFunc_t hash);
void map_free(map_t *tbl);
void *map_get(map_t *tbl, void *key, size_t keysz);
size_t map_getsz(map_t *tbl, void *key, size_t keysz);
void map_put(map_t *tbl, void *key, size_t keysz, void *val, size_t valsz);
void map_del(map_t *tbl, void *key, size_t keysz);
float map_load(map_t *map);

#endif
