#include <stdint.h>

typedef struct { volatile uint8_t *ptr; } Spi_t;

typedef struct { uint32_t bits; } SpiCr1_t;

static inline SpiCr1_t spi_cr1_read(Spi_t ptr) {
    return { *(volatile uint32_t*)(ptr.ptr) };
}

static inline void spi_cr1_write(Spi_t ptr, SpiCr1_t val) {
    *(volatile uint32_t*)(ptr.ptr) = val.bits;
}

#define SPI_CR1_UPDATE(ptr, scope) \
    do { \
        SpiCr1_t reg = spi_cr1_read(ptr); \
        do {scope} while(0); \
        spi_cr1_write(ptr, reg); \
    } while(0)

static inline uint8_t spi_cr1_spe(SpiCr1_t reg) {
    return (reg.bits >> 0) & 1;
}

static inline void spi_cr1_set_spe(SpiCr1_t *reg, uint8_t val) {
    // TODO
}