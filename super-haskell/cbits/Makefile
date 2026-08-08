# Super Haskell C Backend Makefile
# Builds cryptographic primitives and WORM chain implementation
#
# Targets:
#   all      - Build static library
#   test     - Run unit tests
#   bench    - Run benchmarks
#   clean    - Remove build artifacts
#
# Author: Ahmad Parr <ahmedparr93@gmail.com>

CC = gcc
AS = nasm
AR = ar
CFLAGS = -O3 -march=native -Wall -Wextra -std=c11 -fPIC
ASFLAGS = -f elf64
LDFLAGS = -shared

# Detect CPU features
HAS_BMI2 := $(shell $(CC) -march=native -dM -E - < /dev/null | grep -q BMI2 && echo 1 || echo 0)
HAS_ADX := $(shell $(CC) -march=native -dM -E - < /dev/null | grep -q ADX && echo 1 || echo 0)
HAS_AVX512 := $(shell $(CC) -march=native -dM -E - < /dev/null | grep -q AVX512F && echo 1 || echo 0)

ifeq ($(HAS_BMI2),1)
CFLAGS += -DHAS_BMI2
endif

ifeq ($(HAS_ADX),1)
CFLAGS += -DHAS_ADX
endif

ifeq ($(HAS_AVX512),1)
CFLAGS += -DHAS_AVX512
endif

# Source files
C_SOURCES = crypto.c worm_chain.c
ASM_SOURCES_X86 = asm/scalar_mult_x86_64.s
ASM_SOURCES_ARM = asm/scalar_mult_arm.s

# Object files
C_OBJECTS = $(C_SOURCES:.c=.o)
ASM_OBJECTS_X86 = $(ASM_SOURCES_X86:.s=.o)
ASM_OBJECTS_ARM = $(ASM_SOURCES_ARM:.s=.o)

# Detect architecture
ARCH := $(shell uname -m)

ifeq ($(ARCH),x86_64)
ASM_OBJECTS = $(ASM_OBJECTS_X86)
AS = nasm
ASFLAGS = -f elf64
else ifeq ($(ARCH),aarch64)
ASM_OBJECTS = $(ASM_OBJECTS_ARM)
AS = as
ASFLAGS = -march=armv8-a
else
ASM_OBJECTS =
endif

ALL_OBJECTS = $(C_OBJECTS) $(ASM_OBJECTS)

# Output library
LIBNAME = libsuperhaskell.a

.PHONY: all test bench clean install

all: $(LIBNAME)

$(LIBNAME): $(ALL_OBJECTS)
	@echo "AR      $@"
	@$(AR) rcs $@ $^
	@echo "Built $@ ($(ARCH) architecture)"

%.o: %.c
	@echo "CC      $<"
	@$(CC) $(CFLAGS) -c $< -o $@

asm/%.o: asm/%.s
	@echo "AS      $<"
	@$(AS) $(ASFLAGS) $< -o $@

# Tests
test: $(LIBNAME)
	@echo "Building tests..."
	@$(CC) $(CFLAGS) -o test_crypto test_crypto.c $(LIBNAME) -lm
	@echo "Running tests..."
	@./test_crypto

# Benchmarks
bench: $(LIBNAME)
	@echo "Building benchmarks..."
	@$(CC) $(CFLAGS) -o bench_crypto bench_crypto.c $(LIBNAME) -lm
	@echo "Running benchmarks..."
	@./bench_crypto

# Clean
clean:
	@echo "Cleaning..."
	@rm -f $(C_OBJECTS) $(ASM_OBJECTS_X86) $(ASM_OBJECTS_ARM) $(LIBNAME)
	@rm -f test_crypto bench_crypto
	@echo "Clean complete"

# Install
install: $(LIBNAME)
	@echo "Installing to /usr/local/lib..."
	@install -m 644 $(LIBNAME) /usr/local/lib/
	@install -m 644 crypto.h /usr/local/include/
	@install -m 644 worm_chain.h /usr/local/include/
	@echo "Install complete"

# Print configuration
config:
	@echo "Configuration:"
	@echo "  Architecture: $(ARCH)"
	@echo "  CC: $(CC)"
	@echo "  CFLAGS: $(CFLAGS)"
	@echo "  AS: $(AS)"
	@echo "  ASFLAGS: $(ASFLAGS)"
	@echo "  BMI2 support: $(HAS_BMI2)"
	@echo "  ADX support: $(HAS_ADX)"
	@echo "  AVX-512 support: $(HAS_AVX512)"
	@echo "  ASM objects: $(ASM_OBJECTS)"
