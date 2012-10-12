.PHONY: all
all: libamulet

.PHONY: libamulet
libamulet:
	rustc amulet/amulet.rc


DEMO_SOURCES := $(wildcard demos/*.rs)
DEMO_TARGETS := $(DEMO_SOURCES:.rs=)

.PHONY: demos
demos: $(DEMO_TARGETS)

.SECONDEXPANSION:
$(DEMO_TARGETS): libamulet $$@.rs
	rustc -L amulet $@.rs


.PHONY: clean
clean:
	rm -f amulet/libamulet-*.so
	rm -f $(DEMO_TARGETS)
