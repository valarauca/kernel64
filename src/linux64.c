
#include <stdint.h>
#include <linux/random.h>

uint64_t simple_rand(uint8_t *buf, uint64_t len)
{
	return getrandom(buf,len,GRND_RANDOM);
}

uint64_t simple_urand(uint8_t *buf, uint64_t len)
{
	return getrandom(buf,len,GRND_NONBLOCK);
}
