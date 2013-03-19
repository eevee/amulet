RUSTC := rustc

.PHONY: all
all: libamulet demos

.PHONY: libamulet
libamulet: amulet/.built

# need to use a dummy file because rustc spits out .so files with hashed and
# thus basically useless names
amulet/.built: amulet/amulet.rc $(wildcard amulet/*.rs)
	$(RUSTC) amulet/amulet.rc \
	    && touch amulet/.built


DEMO_SOURCES := $(wildcard demos/*.rs)
DEMO_TARGETS := $(DEMO_SOURCES:.rs=)

.PHONY: demos
demos: $(DEMO_TARGETS)

demos/% :: demos/%.rs amulet/.built
	$(RUSTC) -L amulet $@.rs


.PHONY: clean
clean:
	rm -f amulet/libamulet*.so
	rm -f $(DEMO_TARGETS)
