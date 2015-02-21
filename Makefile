RUSTC := rustc

.PHONY: all
all: libamulet.rlib demos

libamulet.rlib: amulet/amulet.rc $(wildcard amulet/*.rs)
	$(RUSTC) amulet/amulet.rc \
	    && touch amulet/.built


DEMO_SOURCES := $(sort $(wildcard demos/*.rs))
DEMO_TARGETS := $(DEMO_SOURCES:.rs=)

.PHONY: demos
demos: $(DEMO_TARGETS)

demos/%: demos/%.rs libamulet.rlib
	$(RUSTC) -L . --out-dir demos $@.rs


.PHONY: clean
clean:
	rm -f amulet/libamulet*.so
	rm -f amulet/.built
	rm -f $(DEMO_TARGETS)
