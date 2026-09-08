/**
 * @file      gc3processingstatelabel.h
 * @brief     The indicator of the processing state of an input group.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GC3PROCESSINGSTATELABEL_H
#define GC3PROCESSINGSTATELABEL_H

#include "gc3multistatelabel.h"

namespace genc3wb::widget
{

/**
 * @brief   Initialises the resources of *product*.
 * @details * The resources are compiled into a static library, and a static library contributes an object
 *            file only where something references it. Without this call the style sheets of the indicator
 *            name images that do not resolve, and the indicator draws nothing while reporting nothing.
 *          * Calling it more than once is harmless.
 */
void initResources();


// realises FR-018, FR-019, FR-020, FR-021
/**
 * @brief   The processing state of an input group.
 * @details * The order of the enumerators is the order of the images the indicator shows.
 */
enum class ProcessingState
{
    ValidFileTextUntouched   = 0,   ///< the named file exists and the text does not differ from it
    ValidFileTextChanged     = 1,   ///< the named file exists and the text differs from it
    UnknownFileTextUntouched = 2,   ///< it is not established that the file exists, and the text was not edited since
    UnknownFileTextChanged   = 3    ///< it is not established that the file exists, and the text was edited since
};

// realises FR-017
/**
 * @brief   Shows the processing state of an input group as one of four images.
 * @details * One image per enumerator of ProcessingState, in that order.
 * @par prefix wdgt
 */
class Gc3ProcessingStateLabel : public Gc3MultiStateLabel
{
    Q_OBJECT

public: // constructors / destructors
    explicit Gc3ProcessingStateLabel(QWidget* pwdgtParent = nullptr);
    virtual ~Gc3ProcessingStateLabel();

public: // methods
    /** @brief Shows the image of that processing state. */
    virtual void setProcessingState(ProcessingState eState);

    /**
     * @brief   Yields the processing state shown.
     * @return  the state shown, ValidFileTextUntouched where none was shown yet
     */
    virtual ProcessingState eProcessingState() const;
};

}

#endif // GC3PROCESSINGSTATELABEL_H
