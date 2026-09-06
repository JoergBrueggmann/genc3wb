/**
 * @file      test.cpp
 * @brief     The root of the test suite.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#include "test.h"

#include "neatest.h"
#include "test_settings.h"
#include "test_widget.h"

bool test()
{
    TEST_INIT();
    TEST( widget );
    TEST( settings );
    TEST_RETURN();
}
