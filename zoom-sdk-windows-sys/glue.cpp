#include "glue.hpp"
#include <iostream>

using namespace ZOOMSDK;

void StringDrop(wchar_t *string) {
    delete string;
}
