CLEANUP = rm -f
MKDIR = mkdir -p
TARGET_EXTENSION=out

.PHONY: clean
.PHONY: test
.PHONY: compile

PATHU = ../Unity/src/
PATHS = src/
PATHT = test/

PATHB = build/
PATHD = build/depends/
PATHO = build/objs/
PATHR = build/results/
OBJS = $(patsubst %,$(PATHO)%,$(_OBJS))

BUILD_PATHS = $(PATHB) $(PATHD) $(PATHO) $(PATHR)

SRCT = $(wildcard $(PATHT)*.c)

COMPILE=gcc -c
LINK=gcc
DEPEND=gcc -MM -MG -MF
CFLAGS=-I. -I$(PATHU) -I$(PATHS) -DTEST
_OBJS = hack_memory.o hack_alu.o hack_pc.o hack_executor.o

RESULTS = $(patsubst $(PATHT)test_%.c,$(PATHR)test_%.txt,$(SRCT) )

PASSED = `grep -s PASS $(PATHR)*.txt`
FAIL = `grep -s FAIL $(PATHR)*.txt`
IGNORE = `grep -s IGNORE $(PATHR)*.txt`

all: test compile

test: $(PATHO)unity.o $(BUILD_PATHS) $(RESULTS)
	@echo "-----------------------\nIGNORES:\n-----------------------"
	@echo "$(IGNORE)"
	@echo "-----------------------\nFAILURES:\n-----------------------"
	@echo "$(FAIL)"
	@echo "-----------------------\nPASSED:\n-----------------------"
	@echo "$(PASSED)"
	@echo "\nDONE"

$(PATHR)%.txt: $(PATHB)%.$(TARGET_EXTENSION)
	-./$< > $@ 2>&1

$(PATHB)test_hack_alu.$(TARGET_EXTENSION): $(PATHO)test_hack_alu.o $(PATHO)hack_memory.o $(PATHO)hack_alu.o
	$(LINK) -o $@ $(PATHO)unity.o $^

$(PATHB)test_hack_executor.$(TARGET_EXTENSION): $(PATHO)test_hack_executor.o $(PATHO)hack_memory.o $(PATHO)hack_alu.o $(PATHO)hack_pc.o $(PATHO)hack_executor.o
	$(LINK) -o $@ $(PATHO)unity.o $^

$(PATHB)test_%.$(TARGET_EXTENSION): $(PATHO)test_%.o $(PATHO)%.o
	$(LINK) -o $@ $(PATHO)unity.o $^

$(PATHO)%.o:: $(PATHT)%.c
	$(COMPILE) $(CFLAGS) $< -o $@

$(PATHO)%.o:: $(PATHS)%.c
	$(COMPILE) $(CFLAGS) $< -o $@

$(PATHO)%.o:: $(PATHU)%.c $(PATHU)%.h
	$(COMPILE) $(CFLAGS) $< -o $@

$(PATHD)%.d:: $(PATHT)%.c
	$(DEPEND) $@ $<

$(PATHB):
	$(MKDIR) $(PATHB)

$(PATHD):
	$(MKDIR) $(PATHD)

$(PATHO):
	$(MKDIR) $(PATHO)

$(PATHR):
	$(MKDIR) $(PATHR)

compile: $(OBJS)

clean:
	$(CLEANUP) $(PATHO)*.o
	$(CLEANUP) $(PATHB)*.$(TARGET_EXTENSION)
	$(CLEANUP) $(PATHR)*.txt
	$(CLEANUP) hack_executor.$(TARGET_EXTENSION)

.PRECIOUS: $(PATHB)test_%.$(TARGET_EXTENSION)
.PRECIOUS: $(PATHD)%.d
.PRECIOUS: $(PATHO)%.o
.PRECIOUS: $(PATHR)%.txt
