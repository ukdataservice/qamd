
STATIC_LIB := ReadStat/src/libReadStat.a
WIN_STATIC_LIB := ReadStat/src/ReadStat.lib

SOURCES := $(wildcard ReadStat/src/*.c ReadStat/src/sas/*.c ReadStat/src/spss/*.c ReadStat/src/stata/*.c)
OBJECTS = $(SOURCES:.c=.o)

CC := clang
INC := -IReadStat
CCFLAGS := -DNDEBUG $(INC) -DHAVE_ZLIB -g -O2 -fPIC -Wall -std=c99

.PHONY: all windows clean

all : clean $(OBJECTS)
	ar rcs $(STATIC_LIB) $(OBJECTS)

windows : clean $(OBJECTS)
	ar rcs $(WIN_STATIC_LIB) $(OBJECTS)

%.o : %.c
	$(CC) $(CCFLAGS) -c $< -o $@

clean :
	-rm $(OBJECTS) $(STATIC_LIB) -f 2> /dev/null

