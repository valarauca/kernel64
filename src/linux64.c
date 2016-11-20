
#include <stdint.h>
#include <linux/random.h>
#include <sys/mman.h>
#include <sys/uio.h>

/*
 * Define front ends to the `getrandom` C function exposed by the
 * linux kernel
 */
uint64_t simple_rand(uint8_t *buf, uint64_t len)
{
	return getrandom(buf,len,GRND_RANDOM);
}
uint64_t simple_urand(uint8_t *buf, uint64_t len)
{
	return getrandom(buf,len,GRND_NONBLOCK);
}

/*
 *Define front ends for a few useful mmap functions
 *
 */
#define RW PROT_READ|PROT_WRITE
uint64_t cow(uint8_t *buf, uint64_t len)
{
	return mmap(buf,len,RW,MAP_PRIVATE,0,0);
}
uint64_t memmap_ro(int32_t fd, uint64_t len) {
	return mmap(0,len,PROT_READ,MAP_POPULATE,fd,0);
}
uint64_t memap_rw(int32_t fd, uint64_t len) {
	return mmap(0,len,RW,MAP_POPULATE,fd,0);
}

/*
 *Define front ends for the process_vm_read/write
 *
 */
uint64_t vm_read(int32_t pid,
		uint8_t *local_,
		uint64_t local_len,
		uint8_t *remote_,
		uint64_t remote_len)
{
	iovec local,remote;
	local.iovec_base = local_;
	local.iovec_len = local_len;
	remote.iovec_base = remote_;
	remote.iovec_len = remote_len;
	return process_vm_readv(pid,&local,1,&remote,1,0);
}
uint64_t vm_write(int32_t pid,
		uint8_t *local_,
		uint64_t local_len,
		uint8_t *remote_,
		uint64_t remote_len)
{
	iovec local,remote;
	local.iovec_base = local_;
	local.iovec_len = local_len;
	remote.iovec_base = remote_;
	remote.iovec_len = remote_len;
	return process_vm_writev(pid,&local,1,&remote,1,0);
}

