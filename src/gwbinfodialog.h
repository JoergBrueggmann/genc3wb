/**
 * @file      gwbinfodialog.h
 * @brief     The modal dialog that states what product is.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GWBINFODIALOG_H
#define GWBINFODIALOG_H

#include <QDialog>
#include <QDialogButtonBox>

namespace genc3wb::mainwindow
{

// realises FR-003, FR-004
/**
 * @brief   The modal dialog that states the name, the version and the author of product.
 * @details * The dialog is modal, so that the main window is reached again only after it is closed.
 *          * Closing it returns the focus to the window that opened it.
 * @par prefix wnd
 */
class GwbInfoDialog : public QDialog
{
    Q_OBJECT

public: // constructors / destructors
    explicit GwbInfoDialog(QWidget* pwdgtParent = nullptr);
    virtual ~GwbInfoDialog();

public: // methods
    /**
     * @brief   Yields the button box by which the dialog is closed.
     * @return  the button box, never null
     */
    virtual QDialogButtonBox* pwdgtButtonBox() const;

protected: // attributes
    QDialogButtonBox*   m_pwdgtButtonBox;   ///< the box carrying the button that closes the dialog
};

}

#endif // GWBINFODIALOG_H
