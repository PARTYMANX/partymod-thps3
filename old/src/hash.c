#include "hash.h"

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>

// possible TODO: load factor

// fnv-1a hash
#define FNV32_OFFSET 2166136261
#define FNV32_PRIME 16777619
uint32_t memhash(void *p, size_t sz) {
	uint32_t result = FNV32_OFFSET; 

	for (uint8_t *i = p; i < (uint8_t *)p + sz; i++) {
		result ^= *i;
		result *= FNV32_PRIME;
	}
	
	return result;
}

// crc32
uint32_t memcrc(void *p, size_t sz) {
	return 0;
}

// map

map_t *map_alloc(size_t len, compareFunc_t cmp, hashFunc_t hash) {
	map_t *result = malloc(sizeof(map_t));

	result->cmp = (cmp) ? cmp : memcmp;
	result->hash = (hash) ? hash : memhash;

	result->len = len;

	result->entries = 0;

	result->buckets = calloc(len, sizeof(struct bucket));

	return result;
}

void map_free(map_t *tbl) {
	for (struct bucket *i = tbl->buckets; i < tbl->buckets + tbl->len; i++) {	// 0x0000017ec115a010, 0x0000017ec115a020
		struct bucketNode *j = i->head;
		while (j) {
			free(j->key);
			free(j->val);
			struct bucketNode *tmp = j;
			j = j->next;
			free(tmp);
		}
	}

	free(tbl->buckets);
	free(tbl);
}

float map_load(map_t *map) {
	return ((float)map->entries) / ((float)map->len);
}

void *map_get(map_t *tbl, void *key, size_t keysz) {
	uint32_t hash = tbl->hash(key, keysz);
	hash = hash % tbl->len;

	struct bucketNode *i = tbl->buckets[hash].head;
	while (i && (keysz != i->keysz || tbl->cmp(key, i->key, keysz) != 0)) {
		i = i->next;
	}

	if (!i) {
		return NULL;
	}

	return i->val;
}

size_t map_getsz(map_t *tbl, void *key, size_t keysz) {
	uint32_t hash = tbl->hash(key, keysz);
	hash = hash % tbl->len;

	struct bucketNode *i = tbl->buckets[hash].head;
	while (i && (keysz != i->keysz || tbl->cmp(key, i->key, keysz) != 0)) {
		i = i->next;
	}

	if (!i) {
		return NULL;
	}

	return i->valsz;
}

void map_put(map_t *tbl, void *key, size_t keysz, void *val, size_t valsz) {
	uint32_t hash = tbl->hash(key, keysz);
	hash = hash % tbl->len;

	struct bucketNode *i = tbl->buckets[hash].head;
	while (i && (keysz != i->keysz || tbl->cmp(key, i->key, keysz) != 0)) {
		i = i->next;
	}

	struct bucketNode *n;

	if (!i) {
		// didn't find key, create
		i = malloc(sizeof(struct bucketNode));
		//n->sz = sz;
		i->keysz = keysz;
		i->key = malloc(keysz);
		memcpy(i->key, key, keysz);
		i->next = tbl->buckets[hash].head;

		tbl->buckets[hash].head = i;

		tbl->entries++;
		tbl->buckets[hash].len++;
	} else {
		// found key, delete current value
		free(i->val);
	}

	i->valsz = valsz;
	i->val = malloc(valsz);
	memcpy(i->val, val, valsz);
}

void map_del(map_t *tbl, void *key, size_t keysz) {
	uint32_t hash = tbl->hash(key, keysz);
	hash = hash % tbl->len;

	// find the entry
	struct bucketNode **i = &(tbl->buckets[hash].head);
	while ((*i) && (keysz != (*i)->keysz || tbl->cmp(key, (*i)->key, keysz) != 0)) {
		(*i) = (*i)->next;
	}

	if (*i) {
		struct bucketNode *tmp = *i;
		*i = tmp->next;
		free(tmp->key);
		free(tmp->val);
		free(tmp);

		tbl->entries--;
	}
}
