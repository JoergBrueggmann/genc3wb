/**
 * @file      gwbmainwindow.h
 * @brief     The main window of the workbench.
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */

#ifndef GWBMAINWINDOW_H
#define GWBMAINWINDOW_H

#include "gc3codeeditorwidget.h"
#include "gc3lineedit.h"
#include "gc3processingstatelabel.h"
#include "mdlsettings.h"

#include <QGroupBox>
#include <QLabel>
#include <QMainWindow>
#include <QMenu>
#include <QPushButton>
#include <QStackedWidget>
#include <QToolButton>

namespace genc3wb::mainwindow
{

/**
 * @brief   The widgets of one input group.
 * @details * One such group edits the compiler-compiler input file, the other the compiler input file.
 * @par prefix grp
 */
struct InputGroup
{
    QGroupBox*                                  pwdgtGroupBox;      ///< the group box holding the widgets below
    genc3wb::widget::Gc3LineEdit*               pwdgtFileName;      ///< the path of the input file
    QToolButton*                                pwdgtFileSelector;  ///< opens the file selector of this group
    genc3wb::widget::Gc3ProcessingStateLabel*   pwdgtIndicator;     ///< the processing state of this group
    genc3wb::widget::Gc3CodeEditorWidget*       pwdgtCodeEditor;    ///< the text of the input file
};

// realises FR-001, FR-002, FR-005, FR-006
/**
 * @brief   The main window, composed of the two input groups, the compiler-compiler group and the output group.
 * @details * The widgets are laid out here; what drives them belongs to the controllers of the other components.
 *          * The window is built in code rather than from a file a designer tool generates, so that the widget
 *            tree of the design is readable in one place.
 * @par prefix wnd
 */
class GwbMainWindow : public QMainWindow
{
    Q_OBJECT

public: // constructors / destructors
    explicit GwbMainWindow(QWidget* pwdgtParent = nullptr);
    virtual ~GwbMainWindow();

public: // information
    /**
     * @brief   Yields the widgets of one input group.
     * @param   eKind   which input file the group edits
     * @return  the widgets of that group
     */
    virtual const InputGroup& groupOfKind(genc3wb::settings::InputKind eKind) const;

    /** @brief Yields the group box of the compiler-compiler. */
    virtual QGroupBox* pwdgtGroupBoxCC() const;

    /** @brief Yields the group box of the output. */
    virtual QGroupBox* pwdgtGroupBoxCOut() const;

    /** @brief Yields the help menu. */
    virtual QMenu* pwdgtMenuHelp() const;

public slots:
    // realises FR-003
    /** @brief Presents the modal info dialog. */
    virtual void on_actionInfo_triggered();

    // realises FR-006
    /**
     * @brief   Notifies the widget that lost the focus and the widget that received it.
     * @param   pwdgtOld    the widget that lost the focus, null where there was none
     * @param   pwdgtNow    the widget that received it, null where none did
     */
    virtual void on_focusChanged(QWidget* pwdgtOld, QWidget* pwdgtNow);

protected: // internal methods
    /** @brief Builds one input group and yields its widgets. */
    virtual InputGroup buildInputGroup(const QString& strTitle, const QString& strSuffix, QWidget* pwdgtParent);

    /** @brief Builds the group of the compiler-compiler. */
    virtual void buildCompilerCompilerGroup(QWidget* pwdgtParent);

    /** @brief Builds the output group with its pages. */
    virtual void buildOutputGroup(QWidget* pwdgtParent);

    /** @brief Builds the menu bar and the status bar. */
    virtual void buildMenuAndStatusBar();

protected: // attributes
    InputGroup      m_groupCCInp;               ///< the group of the compiler-compiler input file
    InputGroup      m_groupCInp;                ///< the group of the compiler input file

    QGroupBox*      m_pwdgtGroupBoxCC;          ///< the group of the compiler-compiler
    genc3wb::widget::Gc3LineEdit*             m_pwdgtLineEditCC;      ///< the path of the compiler-compiler
    QToolButton*                              m_pwdgtToolButtonCC;    ///< opens its file selector
    genc3wb::widget::Gc3ProcessingStateLabel* m_pwdgtIndicatorCC;     ///< whether it can be run

    QGroupBox*      m_pwdgtGroupBoxCOut;        ///< the output group
    QStackedWidget* m_pwdgtStackedWidget;       ///< holds the output pages
    QPushButton*    m_pwdgtPushButtonLeft;      ///< presents the previous page
    QPushButton*    m_pwdgtPushButtonRight;     ///< presents the next page
    QPushButton*    m_pwdgtPushButtonCOut;      ///< detaches the output group
    QLabel*         m_pwdgtLabelPage;           ///< which page of how many is presented

    QMenu*          m_pwdgtMenuHelp;            ///< the help menu
};

}

#endif // GWBMAINWINDOW_H
