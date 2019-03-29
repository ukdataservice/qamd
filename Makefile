
STATIC_LIB := ReadStat/src/libReadStat.a

SOURCES := $(wildcard ReadStat/src/*.c ReadStat/src/sas/*.c ReadStat/src/spss/*.c ReadStat/src/stata/*.c)
OBJECTS = $(SOURCES:.c=.o)

CC := clang
INC := -I/usr/local/include -IReadStat
CCFLAGS := -DNDEBUG $(INC) -DHAVE_ZLIB -g -O2 -Wall -std=c99

.PHONY: all clean

all : clean $(OBJECTS)
	libtool -static -o $(STATIC_LIB) $(OBJECTS)

%.o : %.c
	$(CC) $(CCFLAGS) -c $< -o $@

clean :
	-rm ReadStat/src/*.o ReadStat/src/sas/*.o ReadStat/src/spss/*.o ReadStat/src/stata/*.o $(STATIC_LIB) -f 2> /dev/null

