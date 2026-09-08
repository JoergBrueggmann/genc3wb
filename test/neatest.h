/**
 * @file      neatest.h
 * @brief     The macro framework of the test suite.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef NEATEST_H
#define NEATEST_H

#include <iostream>
using std::cout;

/** @brief Declares the success flag of a test group or of an asserting test case. */
#define TEST_INIT()                                                         \
    bool    _test_bSuccess = true;

/** @brief Runs a test case or test group, while the enclosing group has not failed. */
#define TEST( NAME )                                                        \
    if ( _test_bSuccess )                                                   \
    {                                                                       \
        _test_bSuccess = Test ## NAME();                                    \
        cout                                                                \
            << #NAME " - \ttest result = "                                  \
            << (_test_bSuccess ? "success\n" : "failed\n");                 \
        if ( ! _test_bSuccess )                                             \
        {                                                                   \
            cout << "   file:" << __FILE__ << "\n";                         \
            cout << "   line:" << __LINE__ << "\n";                         \
        }                                                                   \
    }

/** @brief Records a failure where the expression is false, naming the expression, file and line. */
#define TEST_ASSERT( EXPRESSION )                                           \
    if ( ! ( EXPRESSION ) )                                                 \
    {                                                                       \
        _test_bSuccess = false;                                             \
        cout                                                                \
            << "   assertion failed: " #EXPRESSION "\n"                     \
            << "   file:" << __FILE__ << "\n"                               \
            << "   line:" << __LINE__ << "\n";                              \
    }

/** @brief Declares or defines a test case or a test group. */
#define TEST_DEF( NAME )                                                    \
    bool    Test ## NAME()

/** @brief Declares a test case a friend of the class under test. */
#define TEST_FRIEND( NAME )                                                 \
    friend  bool    Test ## NAME()

/** @brief Returns the success flag. */
#define TEST_RETURN()                                                       \
    return ( _test_bSuccess )

/** @brief The success flag itself. */
#define TEST_SUCCESS    _test_bSuccess

#endif // NEATEST_H
