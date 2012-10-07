all: libamulet

libamulet:
	rustc amulet/amulet.rc


clean:
	rm -f amulet/libamulet-*.so
