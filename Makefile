.PHONY: all
all: libamulet

# TODO this is built every time, but the output filename varies...
.PHONY: libamulet
libamulet: amulet/amulet.rc $(wildcard amulet/*.rs)
	rustc amulet/amulet.rc


DEMO_SOURCES := $(wildcard demos/*.rs)
DEMO_TARGETS := $(DEMO_SOURCES:.rs=)

.PHONY: demos
demos: $(DEMO_TARGETS)

.SECONDEXPANSION:
$(DEMO_TARGETS): $$@.rs | libamulet
	rustc -L amulet $@.rs


.PHONY: clean
clean:
	rm -f amulet/libamulet-*.so
	rm -f $(DEMO_TARGETS)
