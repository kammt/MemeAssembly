/*
This file is part of the MemeAssembly compiler.

 Copyright © 2021-2023 Tobias Kamm and contributors

MemeAssembly is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

MemeAssembly is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with MemeAssembly. If not, see <https://www.gnu.org/licenses/>.
*/
#include <stdio.h>

extern void toupper_str(char* str);

int main() {
    char str[] = "The qu1ck BR0wN FoX juMpZ ov3r ThE l4zY dog. Hel1, Y3aH!";
    printf("Before toupper_str: %s\n", str);
    toupper_str(str);
    printf("After toupper_str: %s\n", str);
}    
